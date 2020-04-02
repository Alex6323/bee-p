//TODO:
//Create Readme
//Tests - sanity and multithreaded + benchmarking
//Support multiple sql backends via sqlx
//Get rid of all warnings

extern crate bincode;
extern crate bytemuck;

mod errors;
mod statements;
mod test;

use bytemuck::cast_slice;

use sqlx::Error as SqlxError;

use errors::SqlxBackendError;

use crate::storage::{
    Connection,
    HashesToApprovers,
    MissingHashesToRCApprovers,
    StateDeltaMap,
    Storage,
    StorageBackend,
};

use bee_bundle::{
    Address,
    Hash,
    Index,
    Nonce,
    Payload,
    Tag,
    Timestamp,
    TransactionField,
    Value,
    ADDRESS_TRIT_LEN,
    HASH_TRIT_LEN,
    NONCE_TRIT_LEN,
    PAYLOAD_TRIT_LEN,
    TAG_TRIT_LEN,
};
use bee_protocol::{
    Milestone,
    MilestoneIndex,
};
use bee_ternary::{
    T1B1Buf,
    T5B1Buf,
    TritBuf,
    Trits,
    T5B1,
};

use std::collections::{
    HashMap,
    HashSet,
};

use std::{
    rc::Rc,
    slice,
    vec,
};

use async_trait::async_trait;
use futures::executor::block_on;
use sqlx::{
    postgres::PgQueryAs,
    FromRow,
    PgPool,
    Row,
};

use crate::sqlx::statements::*;

struct TransactionWrapper(bee_bundle::Transaction);
struct MilestoneWrapper(Milestone);
struct StateDeltaWrapper(StateDeltaMap);
struct AttachmentData {
    hash: Hash,
    trunk: Hash,
    branch: Hash,
}

const FAILED_ESTABLISHING_CONNECTION: &str = "failed to establish connection.";
const CONNECTION_NOT_INITIALIZED: &str = "connection was not established and therefor is uninitialized.";

impl<'a> sqlx::FromRow<'a, sqlx::postgres::PgRow<'a>> for TransactionWrapper {
    fn from_row(row: &sqlx::postgres::PgRow) -> Result<Self, SqlxError> {
        let value: i64 = row.get::<i32, _>(TRANSACTION_COL_VALUE) as i64;
        let index: usize = row.get::<i16, _>(TRANSACTION_COL_CURRENT_INDEX) as usize;
        let last_index: usize = row.get::<i16, _>(TRANSACTION_COL_LAST_INDEX) as usize;
        let attachment_ts: u64 = row.get::<i32, _>(TRANSACTION_COL_ATTACHMENT_TIMESTAMP) as u64;
        let attachment_lbts: u64 = row.get::<i32, _>(TRANSACTION_COL_ATTACHMENT_TIMESTAMP_LOWER) as u64;
        let attachment_ubts: u64 = row.get::<i32, _>(TRANSACTION_COL_ATTACHMENT_TIMESTAMP_UPPER) as u64;
        let timestamp: u64 = row.get::<i32, _>(TRANSACTION_COL_TIMESTAMP) as u64;

        let payload_tritbuf = decode_bytes(
            row.get::<Vec<u8>, _>(TRANSACTION_COL_PAYLOAD).as_slice(),
            PAYLOAD_TRIT_LEN,
        );
        let address_tritbuf = decode_bytes(
            row.get::<Vec<u8>, _>(TRANSACTION_COL_ADDRESS).as_slice(),
            ADDRESS_TRIT_LEN,
        );
        let obs_tag_tritbuf = decode_bytes(
            row.get::<Vec<u8>, _>(TRANSACTION_COL_OBSOLETE_TAG).as_slice(),
            TAG_TRIT_LEN,
        );
        let bundle_tritbuf = decode_bytes(row.get::<Vec<u8>, _>(TRANSACTION_COL_BUNDLE).as_slice(), HASH_TRIT_LEN);
        let trunk_tritbuf = decode_bytes(row.get::<Vec<u8>, _>(TRANSACTION_COL_TRUNK).as_slice(), HASH_TRIT_LEN);
        let branch_tritbuf = decode_bytes(row.get::<Vec<u8>, _>(TRANSACTION_COL_BRANCH).as_slice(), HASH_TRIT_LEN);
        let tag_tritbuf = decode_bytes(row.get::<Vec<u8>, _>(TRANSACTION_COL_TAG).as_slice(), TAG_TRIT_LEN);
        let nonce_tritbuf = decode_bytes(row.get::<Vec<u8>, _>(TRANSACTION_COL_NONCE).as_slice(), NONCE_TRIT_LEN);

        let builder = bee_bundle::TransactionBuilder::new()
            .with_payload(Payload::from_inner_unchecked(payload_tritbuf))
            .with_address(Address::from_inner_unchecked(address_tritbuf))
            .with_value(Value::from_inner_unchecked(value))
            .with_obsolete_tag(Tag::from_inner_unchecked(obs_tag_tritbuf))
            .with_timestamp(Timestamp::from_inner_unchecked(timestamp))
            .with_index(Index::from_inner_unchecked(index))
            .with_last_index(Index::from_inner_unchecked(last_index))
            .with_bundle(Hash::from_inner_unchecked(bundle_tritbuf))
            .with_trunk(Hash::from_inner_unchecked(trunk_tritbuf))
            .with_branch(Hash::from_inner_unchecked(branch_tritbuf))
            .with_tag(Tag::from_inner_unchecked(tag_tritbuf))
            .with_attachment_ts(Timestamp::from_inner_unchecked(attachment_ts))
            .with_attachment_lbts(Timestamp::from_inner_unchecked(attachment_lbts))
            .with_attachment_ubts(Timestamp::from_inner_unchecked(attachment_ubts))
            .with_nonce(Nonce::from_inner_unchecked(nonce_tritbuf));

        // TODO(thibault) handle error!
        let tx = builder.build().unwrap();

        Ok(Self(tx))
    }
}

impl<'a> sqlx::FromRow<'a, sqlx::postgres::PgRow<'a>> for MilestoneWrapper {
    fn from_row(row: &sqlx::postgres::PgRow) -> Result<Self, SqlxError> {
        let hash_tritbuf = decode_bytes(row.get::<Vec<u8>, _>(MILESTONE_COL_HASH).as_slice(), HASH_TRIT_LEN);

        Ok(Self(Milestone {
            hash: Hash::from_inner_unchecked(hash_tritbuf),
            index: row.get::<i32, _>(MILESTONE_COL_ID) as u32,
        }))
    }
}

impl<'a> sqlx::FromRow<'a, sqlx::postgres::PgRow<'a>> for StateDeltaWrapper {
    fn from_row(row: &sqlx::postgres::PgRow) -> Result<Self, SqlxError> {
        let delta_str = row.get::<String, _>(MILESTONE_COL_DELTA);
        let delta: StateDeltaMap = bincode::deserialize(&delta_str.as_bytes()).unwrap();
        Ok(Self(delta))
    }
}

impl<'a> sqlx::FromRow<'a, sqlx::postgres::PgRow<'a>> for AttachmentData {
    fn from_row(row: &sqlx::postgres::PgRow) -> Result<Self, SqlxError> {
        let hash = Hash::from_inner_unchecked(decode_bytes(
            row.get::<Vec<u8>, _>(TRANSACTION_COL_HASH).as_slice(),
            HASH_TRIT_LEN,
        ));

        let trunk = Hash::from_inner_unchecked(decode_bytes(
            row.get::<Vec<u8>, _>(TRANSACTION_COL_TRUNK).as_slice(),
            HASH_TRIT_LEN,
        ));

        let branch = Hash::from_inner_unchecked(decode_bytes(
            row.get::<Vec<u8>, _>(TRANSACTION_COL_BRANCH).as_slice(),
            HASH_TRIT_LEN,
        ));

        Ok(AttachmentData { hash, trunk, branch })
    }
}

//TODO - Encoded data is T5B1, decodeds to T1B1,should be generic
fn decode_bytes(u8_slice: &[u8], num_trits: usize) -> TritBuf {
    let decoded_column_i8_slice: &[i8] = cast_slice(u8_slice);
    unsafe { Trits::<T5B1>::from_raw_unchecked(decoded_column_i8_slice, num_trits).to_buf::<T1B1Buf>() }
}

fn encode_buffer(buffer: TritBuf<T5B1Buf>) -> Vec<u8> {
    cast_slice(buffer.as_i8_slice()).to_vec()
}

#[derive(Clone, Debug)]
pub struct SqlxBackendConnection {
    connection_pool: Option<PgPool>,
}

impl SqlxBackendConnection {
    pub fn new() -> Self {
        Self { connection_pool: None }
    }
}

#[async_trait]
impl Connection<SqlxBackendConnection> for SqlxBackendConnection {
    type StorageError = SqlxBackendError;

    async fn establish_connection(&mut self, url: &str) -> Result<(), SqlxBackendError> {
        let pool = PgPool::builder().max_size(num_cpus::get() as u32).build(url).await?;
        self.connection_pool = Some(pool);

        Ok(())
    }
    async fn destroy_connection(&mut self) -> Result<(), SqlxBackendError> {
        self.connection_pool.as_ref().unwrap().close().await;
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct SqlxBackendStorage(Storage<SqlxBackendConnection>);

impl SqlxBackendStorage {
    pub fn new() -> Self {
        let stor = Storage {
            connection: SqlxBackendConnection::new(),
        };
        SqlxBackendStorage(stor)
    }

    pub async fn establish_connection(&mut self, url: &str) -> Result<(), SqlxBackendError> {
        let _res = self.0.connection.establish_connection(url).await?;
        Ok(())
    }
    pub async fn destroy_connection(&mut self) -> Result<(), SqlxBackendError> {
        let _res = self.0.connection.destroy_connection().await?;
        Ok(())
    }
}

//TODO - handle errors
#[async_trait]
impl StorageBackend for SqlxBackendStorage {
    type StorageError = SqlxBackendError;

    fn map_existing_transaction_hashes_to_approvers(&self) -> Result<HashesToApprovers, SqlxBackendError> {
        let mut pool = self
            .0
            .connection
            .connection_pool
            .as_ref()
            .expect(CONNECTION_NOT_INITIALIZED);

        const MAX_RECORDS_AT_ONCE: i32 = 1000;
        let mut start: i32 = 0;
        let mut end: i32 = MAX_RECORDS_AT_ONCE;

        let mut hash_to_approvers = HashMap::new();

        loop {
            let rows: Vec<AttachmentData> = block_on(
                sqlx::query_as(SELECT_HASH_BRANCH_TRUNK_LIMIT_STATEMENT)
                    .bind(start)
                    .bind(MAX_RECORDS_AT_ONCE)
                    .fetch_all(pool),
            )?;

            for (_, attachment_data) in rows.iter().enumerate() {
                hash_to_approvers
                    .entry(attachment_data.branch.clone())
                    .or_insert(HashSet::new())
                    .insert(attachment_data.hash.clone());
                hash_to_approvers
                    .entry(attachment_data.trunk.clone())
                    .or_insert(HashSet::new())
                    .insert(attachment_data.hash.clone());
            }

            if rows.len() < MAX_RECORDS_AT_ONCE as usize {
                break;
            }

            start = end;
            end += MAX_RECORDS_AT_ONCE;
        }

        Ok(hash_to_approvers)
    }

    fn map_missing_transaction_hashes_to_approvers(
        &self,
        all_hashes: HashSet<bee_bundle::Hash>,
    ) -> Result<MissingHashesToRCApprovers, SqlxBackendError> {
        let mut pool = self
            .0
            .connection
            .connection_pool
            .as_ref()
            .expect(CONNECTION_NOT_INITIALIZED);

        const MAX_RECORDS_AT_ONCE: i32 = 1000;
        let mut start: i32 = 0;
        let end: i32 = MAX_RECORDS_AT_ONCE;

        let mut missing_to_approvers = HashMap::new();
        loop {
            let rows: Vec<AttachmentData> = block_on(
                sqlx::query_as(SELECT_HASH_BRANCH_TRUNK_LIMIT_STATEMENT)
                    .bind(start)
                    .bind(end)
                    .fetch_all(pool),
            )?;

            for (_, attachment_data) in rows.iter().enumerate() {
                let mut optional_approver_rc = None;

                if !all_hashes.contains(&attachment_data.branch) {
                    optional_approver_rc = Some(Rc::<bee_bundle::Hash>::new(attachment_data.hash.clone()));
                    missing_to_approvers
                        .entry(attachment_data.branch.clone())
                        .or_insert(HashSet::new())
                        .insert(optional_approver_rc.clone().unwrap());
                }

                if !all_hashes.contains(&attachment_data.trunk) {
                    let approver_rc: Rc<bee_bundle::Hash> =
                        optional_approver_rc.map_or(Rc::new(attachment_data.hash.clone()), |rc| rc.clone());
                    missing_to_approvers
                        .entry(attachment_data.trunk.clone())
                        .or_insert(HashSet::new())
                        .insert(approver_rc.clone());
                }
            }

            if rows.len() < MAX_RECORDS_AT_ONCE as usize {
                break;
            }

            start += MAX_RECORDS_AT_ONCE;
        }

        Ok(missing_to_approvers)
    }
    //Implement all methods here
    async fn insert_transaction(
        &self,
        tx_hash: bee_bundle::Hash,
        tx: bee_bundle::Transaction,
    ) -> Result<(), SqlxBackendError> {
        let pool = self
            .0
            .connection
            .connection_pool
            .as_ref()
            .expect(CONNECTION_NOT_INITIALIZED);
        let mut conn_transaction = pool.begin().await?;

        sqlx::query(INSERT_TRANSACTION_STATEMENT)
            .bind(encode_buffer(tx.payload().to_inner().encode::<T5B1Buf>()))
            .bind(encode_buffer(tx.address().to_inner().encode::<T5B1Buf>()))
            .bind(*tx.value().to_inner())
            .bind(encode_buffer(tx.obsolete_tag().to_inner().encode::<T5B1Buf>()))
            .bind(*tx.timestamp().to_inner() as i32)
            .bind(*tx.index().to_inner() as i32)
            .bind(*tx.last_index().to_inner() as i32)
            .bind(encode_buffer(tx.bundle().to_inner().encode::<T5B1Buf>()))
            .bind(encode_buffer(tx.trunk().to_inner().encode::<T5B1Buf>()))
            .bind(encode_buffer(tx.branch().to_inner().encode::<T5B1Buf>()))
            .bind(encode_buffer(tx.tag().to_inner().encode::<T5B1Buf>()))
            .bind(*tx.attachment_ts().to_inner() as i32)
            .bind(*tx.attachment_lbts().to_inner() as i32)
            .bind(*tx.attachment_ubts().to_inner() as i32)
            .bind(encode_buffer(tx.nonce().to_inner().encode::<T5B1Buf>()))
            .bind(encode_buffer(tx_hash.to_inner().encode::<T5B1Buf>()))
            .execute(&mut conn_transaction)
            .await?;

        conn_transaction.commit().await?;

        Ok(())
    }

    async fn find_transaction(&self, tx_hash: bee_bundle::Hash) -> Result<bee_bundle::Transaction, SqlxBackendError> {
        let mut pool = self
            .0
            .connection
            .connection_pool
            .as_ref()
            .expect(CONNECTION_NOT_INITIALIZED);

        let _user_id: i32 = 0;
        let tx_wrapper: TransactionWrapper = sqlx::query_as(FIND_TRANSACTION_BY_HASH_STATEMENT)
            .bind(encode_buffer(tx_hash.to_inner().encode::<T5B1Buf>()))
            .fetch_one(&mut pool)
            .await?;

        Ok(tx_wrapper.0)
    }

    async fn update_transactions_set_solid(
        &self,
        transaction_hashes: HashSet<bee_bundle::Hash>,
    ) -> Result<(), SqlxBackendError> {
        let pool = self
            .0
            .connection
            .connection_pool
            .as_ref()
            .expect(CONNECTION_NOT_INITIALIZED);
        let mut conn_transaction = pool.begin().await?;

        transaction_hashes.iter().for_each(|hash| {
            let _ = sqlx::query(UPDATE_SET_SOLID_STATEMENT)
                .bind(encode_buffer(hash.to_inner().encode::<T5B1Buf>()))
                .execute(&mut conn_transaction);
        });

        conn_transaction.commit().await?;

        Ok(())
    }

    async fn update_transactions_set_snapshot_index(
        &self,
        transaction_hashes: HashSet<bee_bundle::Hash>,
        snapshot_index: MilestoneIndex,
    ) -> Result<(), SqlxBackendError> {
        let pool = self
            .0
            .connection
            .connection_pool
            .as_ref()
            .expect(CONNECTION_NOT_INITIALIZED);
        let mut conn_transaction = pool.begin().await?;

        transaction_hashes.iter().for_each(|hash| {
            let _ = sqlx::query(UPDATE_SNAPSHOT_INDEX_STATEMENT)
                .bind(encode_buffer(hash.to_inner().encode::<T5B1Buf>()))
                .bind(snapshot_index as i32)
                .execute(&mut conn_transaction);
        });

        conn_transaction.commit().await?;

        Ok(())
    }

    async fn delete_transactions(
        &self,
        transaction_hashes: &HashSet<bee_bundle::Hash>,
    ) -> Result<(), SqlxBackendError> {
        let pool = self
            .0
            .connection
            .connection_pool
            .as_ref()
            .expect(CONNECTION_NOT_INITIALIZED);
        let mut conn_transaction = pool.begin().await?;

        for hash in transaction_hashes.iter() {
            sqlx::query(DELETE_TRANSACTION_STATEMENT)
                .bind(encode_buffer(hash.to_inner().encode::<T5B1Buf>()))
                .execute(&mut conn_transaction)
                .await?;
        }

        conn_transaction.commit().await?;

        Ok(())
    }

    async fn insert_transactions(
        &self,
        transactions: HashMap<bee_bundle::Hash, bee_bundle::Transaction>,
    ) -> Result<(), Self::StorageError> {
        let pool = self
            .0
            .connection
            .connection_pool
            .as_ref()
            .expect(CONNECTION_NOT_INITIALIZED);
        let mut conn_transaction = pool.begin().await?;

        for (tx_hash, tx) in transactions {
            sqlx::query(INSERT_TRANSACTION_STATEMENT)
                .bind(encode_buffer(tx.payload().to_inner().encode::<T5B1Buf>()))
                .bind(encode_buffer(tx.address().to_inner().encode::<T5B1Buf>()))
                .bind(*tx.value().to_inner())
                .bind(encode_buffer(tx.obsolete_tag().to_inner().encode::<T5B1Buf>()))
                .bind(*tx.timestamp().to_inner() as i32)
                .bind(*tx.index().to_inner() as i32)
                .bind(*tx.last_index().to_inner() as i32)
                .bind(encode_buffer(tx.bundle().to_inner().encode::<T5B1Buf>()))
                .bind(encode_buffer(tx.trunk().to_inner().encode::<T5B1Buf>()))
                .bind(encode_buffer(tx.branch().to_inner().encode::<T5B1Buf>()))
                .bind(encode_buffer(tx.tag().to_inner().encode::<T5B1Buf>()))
                .bind(*tx.attachment_ts().to_inner() as i32)
                .bind(*tx.attachment_lbts().to_inner() as i32)
                .bind(*tx.attachment_ubts().to_inner() as i32)
                .bind(encode_buffer(tx.nonce().to_inner().encode::<T5B1Buf>()))
                .bind(encode_buffer(tx_hash.to_inner().encode::<T5B1Buf>()))
                .execute(&mut conn_transaction)
                .await?;
        }

        conn_transaction.commit().await?;
        Ok(())
    }

    async fn insert_milestone(&self, milestone: Milestone) -> Result<(), SqlxBackendError> {
        let pool = self
            .0
            .connection
            .connection_pool
            .as_ref()
            .expect(CONNECTION_NOT_INITIALIZED);
        let mut conn_transaction = pool.begin().await?;

        sqlx::query(INSERT_MILESTONE_STATEMENT)
            .bind(milestone.index as i32)
            .bind(encode_buffer(milestone.hash.to_inner().encode::<T5B1Buf>()))
            .execute(&mut conn_transaction)
            .await?;

        conn_transaction.commit().await?;

        Ok(())
    }

    async fn find_milestone(&self, milestone_hash: bee_bundle::Hash) -> Result<Milestone, SqlxBackendError> {
        let mut pool = self
            .0
            .connection
            .connection_pool
            .as_ref()
            .expect(CONNECTION_NOT_INITIALIZED);

        let _user_id: i32 = 0;

        let milestone_wrapper: MilestoneWrapper = sqlx::query_as(FIND_MILESTONE_BY_HASH_STATEMENT)
            .bind(encode_buffer(milestone_hash.to_inner().encode::<T5B1Buf>()))
            .fetch_one(&mut pool)
            .await?;
        Ok(milestone_wrapper.0)
    }

    async fn delete_milestones(&self, milestone_hashes: &HashSet<bee_bundle::Hash>) -> Result<(), SqlxBackendError> {
        let pool = self
            .0
            .connection
            .connection_pool
            .as_ref()
            .expect(CONNECTION_NOT_INITIALIZED);
        let mut conn_transaction = pool.begin().await?;

        for hash in milestone_hashes.iter() {
            sqlx::query(DELETE_MILESTONE_BY_HASH_STATEMENT)
                .bind(encode_buffer(hash.to_inner().encode::<T5B1Buf>()))
                .execute(&mut conn_transaction)
                .await?;
        }

        conn_transaction.commit().await?;

        Ok(())
    }

    async fn insert_state_delta(
        &self,
        state_delta: StateDeltaMap,
        index: MilestoneIndex,
    ) -> Result<(), SqlxBackendError> {
        let pool = self
            .0
            .connection
            .connection_pool
            .as_ref()
            .expect(CONNECTION_NOT_INITIALIZED);
        let mut conn_transaction = pool.begin().await?;

        let encoded: Vec<u8> = bincode::serialize(&state_delta)?;

        sqlx::query(STORE_DELTA_STATEMENT)
            .bind(encoded)
            .bind(index as i32)
            .execute(&mut conn_transaction)
            .await?;

        conn_transaction.commit().await?;

        Ok(())
    }

    async fn load_state_delta(&self, index: MilestoneIndex) -> Result<StateDeltaMap, SqlxBackendError> {
        let mut pool = self
            .0
            .connection
            .connection_pool
            .as_ref()
            .expect(CONNECTION_NOT_INITIALIZED);

        let state_delta_wrapper: StateDeltaWrapper = sqlx::query_as(LOAD_DELTA_STATEMENT_BY_INDEX)
            .bind(index as i32)
            .fetch_one(&mut pool)
            .await?;

        Ok(state_delta_wrapper.0)
    }
}

//TODO:
//Create Readme
//Tests - sanity and multithreaded + benchmarking
//Support multiple sql backends via sqlx
//Get rid of all warnings

mod errors;

use bee_bundle::transaction::TransactionField;
use bee_bundle::*;
use bee_storage::{
    Connection,
    Milestone,
    MissingHashesToRCApprovers,
};
use bee_ternary::{
    T1B1Buf,
    T5B1Buf,
    TritBuf,
    Trits,
    T1B1,
    T5B1,
};

use std::collections::{
    HashMap,
    HashSet,
};
use std::env;
use std::rc::Rc;
use std::slice;

use async_trait::async_trait;
use errors::*;
use futures::executor::block_on;
use sqlx::{
    PgPool,
    Query,
    Row,
};

use std::ops::Deref;

use bee_bundle::constants::{
    ADDRESS,
    ADDRESS_TRIT_LEN,
    BRANCH,
    BUNDLE,
    HASH_TRIT_LEN,
    IOTA_SUPPLY,
    NONCE,
    NONCE_TRIT_LEN,
    OBSOLETE_TAG,
    PAYLOAD,
    PAYLOAD_TRIT_LEN,
    TAG,
    TAG_TRIT_LEN,
    TRUNK,
    TRYTE_ZERO,
};

std::include!("../sql/statements.rs");

//TODO - Encoded data is T5B1, decodeds to T1B1,should be generic
fn decode_bytes(u8_slice: &[u8], num_trits: usize) -> TritBuf {
    let decoded_column_i8_slice: &[i8] =
        unsafe { slice::from_raw_parts(u8_slice.as_ptr() as *const i8, u8_slice.len()) };
    Trits::<T5B1>::try_from_raw(decoded_column_i8_slice, num_trits)
        .unwrap()
        .to_buf::<T1B1Buf>()
}

fn encode_buffer(buffer: TritBuf<T5B1Buf>) -> Vec<u8> {
    let i8_slice = buffer.as_i8_slice();
    unsafe { slice::from_raw_parts(i8_slice.as_ptr() as *const u8, i8_slice.len()) }.to_vec()
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
impl bee_storage::Connection<SqlxBackendConnection> for SqlxBackendConnection {
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
pub struct SqlxBackendStorage(bee_storage::Storage<SqlxBackendConnection>);

impl SqlxBackendStorage {
    pub fn new() -> Self {
        let stor = bee_storage::Storage {
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
impl bee_storage::StorageBackend for SqlxBackendStorage {
    type StorageError = SqlxBackendError;

    fn map_existing_transaction_hashes_to_approvers(&self) -> Result<bee_storage::HashesToApprovers, SqlxBackendError> {
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
            let rows = block_on(
                sqlx::query(SELECT_HASH_BRANCH_TRUNK_LIMIT_STATEMENT)
                    .bind(start)
                    .bind(MAX_RECORDS_AT_ONCE)
                    .fetch_all(&mut pool),
            )?;

            for (_, row) in rows.iter().enumerate() {
                hash_to_approvers
                    .entry(Hash::from_tritbuf_unchecked(decode_bytes(
                        row.get::<Vec<u8>, _>(TRANSACTION_COL_BRANCH).as_slice(),
                        HASH_TRIT_LEN,
                    )))
                    .or_insert(HashSet::new())
                    .insert(Hash::from_tritbuf_unchecked(decode_bytes(
                        row.get::<Vec<u8>, _>(TRANSACTION_COL_HASH).as_slice(),
                        HASH_TRIT_LEN,
                    )));
                hash_to_approvers
                    .entry(Hash::from_tritbuf_unchecked(decode_bytes(
                        row.get::<Vec<u8>, _>(TRANSACTION_COL_TRUNK).as_slice(),
                        HASH_TRIT_LEN,
                    )))
                    .or_insert(HashSet::new())
                    .insert(Hash::from_tritbuf_unchecked(decode_bytes(
                        row.get::<Vec<u8>, _>(TRANSACTION_COL_HASH).as_slice(),
                        HASH_TRIT_LEN,
                    )));
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
            let rows = block_on(
                sqlx::query(SELECT_HASH_BRANCH_TRUNK_LIMIT_STATEMENT)
                    .bind(start)
                    .bind(end)
                    .fetch_all(&mut pool),
            )?;

            for (_, row) in rows.iter().enumerate() {
                let branch = Hash::from_tritbuf_unchecked(decode_bytes(
                    row.get::<Vec<u8>, _>(TRANSACTION_COL_BRANCH).as_slice(),
                    HASH_TRIT_LEN,
                ));
                let trunk = Hash::from_tritbuf_unchecked(decode_bytes(
                    row.get::<Vec<u8>, _>(TRANSACTION_COL_TRUNK).as_slice(),
                    HASH_TRIT_LEN,
                ));
                let mut optional_approver_rc = None;

                if !all_hashes.contains(&branch) {
                    optional_approver_rc = Some(Rc::<bee_bundle::Hash>::new(Hash::from_tritbuf_unchecked(
                        decode_bytes(row.get::<Vec<u8>, _>(TRANSACTION_COL_HASH).as_slice(), HASH_TRIT_LEN),
                    )));
                    missing_to_approvers
                        .entry(Hash::from_tritbuf_unchecked(decode_bytes(
                            row.get::<Vec<u8>, _>(TRANSACTION_COL_BRANCH).as_slice(),
                            HASH_TRIT_LEN,
                        )))
                        .or_insert(HashSet::new())
                        .insert(optional_approver_rc.clone().unwrap());
                }

                if !all_hashes.contains(&trunk) {
                    let approver_rc: Rc<bee_bundle::Hash> = optional_approver_rc.map_or(
                        Rc::new(Hash::from_tritbuf_unchecked(decode_bytes(
                            row.get::<Vec<u8>, _>(TRANSACTION_COL_HASH).as_slice(),
                            HASH_TRIT_LEN,
                        ))),
                        |rc| rc.clone(),
                    );
                    missing_to_approvers
                        .entry(Hash::from_tritbuf_unchecked(decode_bytes(
                            row.get::<Vec<u8>, _>(TRANSACTION_COL_TRUNK).as_slice(),
                            HASH_TRIT_LEN,
                        )))
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
            .bind(encode_buffer(tx.payload().into_inner().encode::<T5B1Buf>()))
            .bind(encode_buffer(tx.address().into_inner().encode::<T5B1Buf>()))
            .bind(tx.value().0)
            .bind(encode_buffer(tx.obsolete_tag().into_inner().encode::<T5B1Buf>()))
            .bind(tx.timestamp().0 as i32)
            .bind(tx.index().0 as i32)
            .bind(tx.last_index().0 as i32)
            .bind(encode_buffer(tx.bundle().into_inner().encode::<T5B1Buf>()))
            .bind(encode_buffer(tx.trunk().into_inner().encode::<T5B1Buf>()))
            .bind(encode_buffer(tx.branch().into_inner().encode::<T5B1Buf>()))
            .bind(encode_buffer(tx.tag().into_inner().encode::<T5B1Buf>()))
            .bind(tx.attachment_ts().0 as i32)
            .bind(tx.attachment_lbts().0 as i32)
            .bind(tx.attachment_ubts().0 as i32)
            .bind(encode_buffer(tx.nonce().into_inner().encode::<T5B1Buf>()))
            .bind(encode_buffer(tx_hash.into_inner().encode::<T5B1Buf>()))
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
        let rec = sqlx::query(FIND_TRANSACTION_BY_HASH_STATEMENT)
            .bind(encode_buffer(tx_hash.into_inner().encode::<T5B1Buf>()))
            .fetch_one(&mut pool)
            .await?;

        let value: i64 = rec.get::<i32, _>(TRANSACTION_COL_VALUE) as i64;
        let index: usize = rec.get::<i16, _>(TRANSACTION_COL_CURRENT_INDEX) as usize;
        let last_index: usize = rec.get::<i16, _>(TRANSACTION_COL_LAST_INDEX) as usize;
        let attachment_ts: u64 = rec.get::<i32, _>(TRANSACTION_COL_ATTACHMENT_TIMESTAMP) as u64;
        let attachment_lbts: u64 = rec.get::<i32, _>(TRANSACTION_COL_ATTACHMENT_TIMESTAMP_LOWER) as u64;
        let attachment_ubts: u64 = rec.get::<i32, _>(TRANSACTION_COL_ATTACHMENT_TIMESTAMP_UPPER) as u64;
        let timestamp: u64 = rec.get::<i32, _>(TRANSACTION_COL_TIMESTAMP) as u64;

        let payload_tritbuf = decode_bytes(
            rec.get::<Vec<u8>, _>(TRANSACTION_COL_PAYLOAD).as_slice(),
            PAYLOAD_TRIT_LEN,
        );
        let address_tritbuf = decode_bytes(
            rec.get::<Vec<u8>, _>(TRANSACTION_COL_ADDRESS).as_slice(),
            ADDRESS_TRIT_LEN,
        );
        let obs_tag_tritbuf = decode_bytes(
            rec.get::<Vec<u8>, _>(TRANSACTION_COL_OBSOLETE_TAG).as_slice(),
            TAG_TRIT_LEN,
        );
        let bundle_tritbuf = decode_bytes(rec.get::<Vec<u8>, _>(TRANSACTION_COL_BUNDLE).as_slice(), HASH_TRIT_LEN);
        let trunk_tritbuf = decode_bytes(rec.get::<Vec<u8>, _>(TRANSACTION_COL_TRUNK).as_slice(), HASH_TRIT_LEN);
        let branch_tritbuf = decode_bytes(rec.get::<Vec<u8>, _>(TRANSACTION_COL_BRANCH).as_slice(), HASH_TRIT_LEN);
        let tag_tritbuf = decode_bytes(rec.get::<Vec<u8>, _>(TRANSACTION_COL_TAG).as_slice(), TAG_TRIT_LEN);
        let nonce_tritbuf = decode_bytes(rec.get::<Vec<u8>, _>(TRANSACTION_COL_NONCE).as_slice(), NONCE_TRIT_LEN);

        let mut builder = bee_bundle::TransactionBuilder::new()
            .with_payload(Payload::from_tritbuf_unchecked(payload_tritbuf))
            .with_address(Address::from_tritbuf_unchecked(address_tritbuf))
            .with_value(Value(value))
            .with_obsolete_tag(Tag::from_tritbuf_unchecked(obs_tag_tritbuf))
            .with_timestamp(Timestamp(timestamp))
            .with_index(Index(index))
            .with_last_index(Index(last_index))
            .with_bundle(Hash::from_tritbuf_unchecked(bundle_tritbuf))
            .with_trunk(Hash::from_tritbuf_unchecked(trunk_tritbuf))
            .with_branch(Hash::from_tritbuf_unchecked(branch_tritbuf))
            .with_tag(Tag::from_tritbuf_unchecked(tag_tritbuf))
            .with_attachment_ts(Timestamp(attachment_ts))
            .with_attachment_lbts(Timestamp(attachment_lbts))
            .with_attachment_ubts(Timestamp(attachment_ubts))
            .with_nonce(Nonce::from_tritbuf_unchecked(nonce_tritbuf));

        // TODO(thibault) handle error!
        let tx = builder.build().unwrap();

        Ok(tx)
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
                .bind(encode_buffer(hash.into_inner().encode::<T5B1Buf>()))
                .fetch_one(&mut conn_transaction);
        });

        Ok(())
    }

    async fn update_transactions_set_snapshot_index(
        &self,
        transaction_hashes: HashSet<bee_bundle::Hash>,
        snapshot_index: u32,
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
                .bind(encode_buffer(hash.into_inner().encode::<T5B1Buf>()))
                .bind(snapshot_index as i32)
                .fetch_one(&mut conn_transaction);
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
            let _ = sqlx::query(DELETE_TRANSACTION_STATEMENT)
                .bind(encode_buffer(hash.into_inner().encode::<T5B1Buf>()))
                .fetch_all(&mut conn_transaction)
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
                .bind(encode_buffer(tx.payload().into_inner().encode::<T5B1Buf>()))
                .bind(encode_buffer(tx.address().into_inner().encode::<T5B1Buf>()))
                .bind(tx.value().0)
                .bind(encode_buffer(tx.obsolete_tag().into_inner().encode::<T5B1Buf>()))
                .bind(tx.timestamp().0 as i32)
                .bind(tx.index().0 as i32)
                .bind(tx.last_index().0 as i32)
                .bind(encode_buffer(tx.bundle().into_inner().encode::<T5B1Buf>()))
                .bind(encode_buffer(tx.trunk().into_inner().encode::<T5B1Buf>()))
                .bind(encode_buffer(tx.branch().into_inner().encode::<T5B1Buf>()))
                .bind(encode_buffer(tx.tag().into_inner().encode::<T5B1Buf>()))
                .bind(tx.attachment_ts().0 as i32)
                .bind(tx.attachment_lbts().0 as i32)
                .bind(tx.attachment_ubts().0 as i32)
                .bind(encode_buffer(tx.nonce().into_inner().encode::<T5B1Buf>()))
                .bind(encode_buffer(tx_hash.into_inner().encode::<T5B1Buf>()))
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
            .bind(encode_buffer(milestone.hash.into_inner().encode::<T5B1Buf>()))
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

        let rec = sqlx::query(FIND_MILESTONE_BY_HASH_STATEMENT)
            .bind(encode_buffer(milestone_hash.into_inner().encode::<T5B1Buf>()))
            .fetch_one(&mut pool)
            .await?;

        let hash_tritbuf = decode_bytes(rec.get::<Vec<u8>, _>(MILESTONE_COL_HASH).as_slice(), HASH_TRIT_LEN);

        let milestone = Milestone {
            hash: Hash::from_tritbuf_unchecked(hash_tritbuf),
            index: rec.get::<i32, _>(MILESTONE_COL_ID) as u32,
        };

        Ok(milestone)
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
            let _row = sqlx::query(DELETE_MILESTONE_BY_HASH_STATEMENT)
                .bind(encode_buffer(hash.into_inner().encode::<T5B1Buf>()))
                .fetch_all(&mut conn_transaction)
                .await?;
        }

        conn_transaction.commit().await?;

        Ok(())
    }

    async fn insert_state_delta(
        &self,
        state_delta: bee_storage::StateDeltaMap,
        index: u32,
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

    async fn load_state_delta(&self, index: u32) -> Result<bee_storage::StateDeltaMap, SqlxBackendError> {
        let mut pool = self
            .0
            .connection
            .connection_pool
            .as_ref()
            .expect(CONNECTION_NOT_INITIALIZED);

        let rec = sqlx::query(LOAD_DELTA_STATEMENT_BY_INDEX)
            .bind(index as i32)
            .fetch_one(&mut pool)
            .await?;

        let delta = rec.get::<String, _>(MILESTONE_COL_DELTA);
        let decoded: bee_storage::StateDeltaMap = bincode::deserialize(&delta.as_bytes())?;

        Ok(decoded)
    }
}

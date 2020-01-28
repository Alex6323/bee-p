//pub extern crate storage;
//pub extern crate serde;
//pub extern crate bincode;

pub mod errors;

//TODO:
//Create Readme
//Tests - sanity and multithreaded + benchmarking
//Support multiple sql backends via sqlx
//Get rid of all warnings


use errors::*;

use sqlx::{Row, PgPool};
use storage::{Connection, HashesToApprovers, MissingHashesToRCApprovers, Milestone};
use async_trait::async_trait;
use std::{fmt, env, cmp, error::Error as StdError,  rc::Rc};
use std::collections::{HashMap, HashSet};
use futures::executor::block_on;
use serde::{Serialize, Deserialize};
use std::io::{self, Write};

pub use bundle::*;


std::include!("../sql/statements.rs");

pub struct SqlxBackendConnection {
    connection_pool: Option<PgPool>,
}

impl SqlxBackendConnection {
    pub fn new() -> Self {
        Self {
            connection_pool: None,
        }
    }
}


#[async_trait]
impl storage::Connection<SqlxBackendConnection> for SqlxBackendConnection {

    type StorageError = SqlxBackendError;

    async fn establish_connection(&mut self) -> Result<(), SqlxBackendError> {
        let pool = PgPool::new(&env::var("BEE_DATABASE_URL")?).await.expect(FAILED_ESTABLISHING_CONNECTION);
        self.connection_pool = Some(pool);

        Ok(())
    }
    async fn destroy_connection(&mut self) -> Result<(), SqlxBackendError> {
        self.connection_pool.as_ref().unwrap().close();
        Ok(())
    }
}

pub struct SqlxBackendStorage(storage::Storage<SqlxBackendConnection>);


impl SqlxBackendStorage {

    pub fn new() -> Self {
        let stor = storage::Storage {
            connection: SqlxBackendConnection::new(),
        };
        SqlxBackendStorage(stor)
    }

    pub async fn establish_connection(&mut self) -> Result<(), SqlxBackendError> {
        let res = self.0.connection.establish_connection().await?;
        Ok(())
    }
    pub async fn destroy_connection(&mut self) -> Result<(), SqlxBackendError> {
        let res = self.0.connection.destroy_connection().await?;
        Ok(())
    }
}

//TODO - handle errors
#[async_trait]
impl storage::StorageBackend for SqlxBackendStorage {

    type StorageError = SqlxBackendError;

    fn map_existing_transaction_hashes_to_approvers(
        &self,
    ) -> Result<storage::HashesToApprovers, SqlxBackendError> {

        let mut pool = self.0.connection.connection_pool.as_ref().expect(CONNECTION_NOT_INITIALIZED);

        const MAX_RECORDS_AT_ONCE : i32 = 1000;
        let mut start: i32 = 0;
        let mut end: i32 = MAX_RECORDS_AT_ONCE;

        let mut hash_to_approvers = HashMap::new();

        loop {
            let rows = block_on(sqlx::query(
                SELECT_HASH_BRANCH_TRUNK_LIMIT_STATEMENT
            ).bind(start).bind(end)
                .fetch_all(&mut pool))?;

            for (i, row) in rows.iter().enumerate()  {

                hash_to_approvers.entry(row.get::<String,_>(TRANSACTION_COL_BRANCH).into()).or_insert(HashSet::new()).insert(row.get::<String,_>(TRANSACTION_COL_HASH).into());
                hash_to_approvers.entry(row.get::<String,_>(TRANSACTION_COL_TRUNK).into()).or_insert(HashSet::new()).insert(row.get::<String,_>(TRANSACTION_COL_HASH).into());

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
        &self, all_hashes: HashSet<bundle::Hash>
    ) -> Result<MissingHashesToRCApprovers, SqlxBackendError> {

        let mut pool = self.0.connection.connection_pool.as_ref().expect(CONNECTION_NOT_INITIALIZED);

        const MAX_RECORDS_AT_ONCE : i32 = 1000;
        let mut start: i32 = 0;
        let mut end: i32 = MAX_RECORDS_AT_ONCE;

        let mut missing_to_approvers = HashMap::new();
        loop {
            let rows = block_on(sqlx::query(
                SELECT_HASH_BRANCH_TRUNK_LIMIT_STATEMENT
            ).bind(start).bind(end)
                .fetch_all(&mut pool))?;

            for (i, row) in rows.iter().enumerate()  {

                let branch : bundle::Hash = row.get::<String,_>(TRANSACTION_COL_BRANCH).into();
                let trunk : bundle::Hash = row.get::<String,_>(TRANSACTION_COL_TRUNK).into();
                let mut optional_approver_rc = None;

                if !all_hashes.contains(&branch){
                    optional_approver_rc = Some(Rc::<bundle::Hash>::new(row.get::<String,_>(TRANSACTION_COL_HASH).into()));
                    missing_to_approvers.entry(row.get::<String,_>(TRANSACTION_COL_BRANCH).into()).or_insert(HashSet::new()).insert(optional_approver_rc.clone().unwrap());
                }


                if !all_hashes.contains(&trunk){
                    let approver_rc : Rc<bundle::Hash> = optional_approver_rc.map_or(Rc::new(row.get::<String,_>(TRANSACTION_COL_HASH).into()), |rc| rc.clone());
                    missing_to_approvers.entry(row.get::<String,_>(TRANSACTION_COL_TRUNK).into()).or_insert(HashSet::new()).insert(approver_rc);
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
    async fn insert_transaction(&self, tx_hash: &bundle::Hash, tx: &bundle::Transaction) -> Result<(), SqlxBackendError> {

        let pool = self.0.connection.connection_pool.as_ref().expect(CONNECTION_NOT_INITIALIZED);
        let mut conn_transaction = pool.begin().await?;


        let row = sqlx::query(
            INSERT_TRANSACTION_STATEMENT
        )
        .bind(tx.payload().to_string().as_bytes())
        .bind(tx.address().to_string().as_bytes())
        .bind(tx.value().0)
        .bind(tx.obsolete_tag().to_string().as_bytes())
        .bind(tx.timestamp().0 as i32)
        .bind(tx.index().0 as i32)
        .bind(tx.last_index().0 as i32)
        .bind(tx.bundle_hash().to_string().as_bytes())
        .bind(tx.trunk_hash().to_string().as_bytes())
        .bind(tx.branch_hash().to_string().as_bytes())
        .bind(tx.tag().to_string().as_bytes())
        .bind(tx.attachment_ts().0 as i32)
        .bind(tx.attachment_lbts().0 as i32)
        .bind(tx.attachment_ubts().0 as i32)
        .bind(tx.nonce().to_string().as_bytes())
        .bind(tx_hash.to_string().as_bytes())
        .fetch_one(&mut conn_transaction)
        .await?;

        conn_transaction.commit().await?;

        Ok(())
    }

    async fn find_transaction(&self, tx_hash: &bundle::Hash) -> Result<bundle::Transaction, SqlxBackendError> {

        let mut pool = self.0.connection.connection_pool.as_ref().expect(CONNECTION_NOT_INITIALIZED);


        let user_id : i32 = 0;
        let rec = sqlx::query(
            FIND_TRANSACTION_BY_HASH_STATEMENT
    ).bind(tx_hash.to_string().as_bytes())
            .fetch_one(&mut pool)
            .await?;

        let value: i64 = rec.get::<i32,_>(TRANSACTION_COL_VALUE) as i64;
        let index: usize = rec.get::<i16,_>(TRANSACTION_COL_CURRENT_INDEX) as usize;
        let last_index: usize = rec.get::<i16,_>(TRANSACTION_COL_LAST_INDEX) as usize;
        let attachment_ts: u64 = rec.get::<i32,_>(TRANSACTION_COL_ATTACHMENT_TIMESTAMP) as u64;
        let attachment_lbts: u64 = rec.get::<i32,_>(TRANSACTION_COL_ATTACHMENT_TIMESTAMP_LOWER) as u64;
        let attachment_ubts: u64 = rec.get::<i32,_>(TRANSACTION_COL_ATTACHMENT_TIMESTAMP_UPPER) as u64;
        let timestamp: u64 = rec.get::<i32,_>(TRANSACTION_COL_TIMESTAMP) as u64;

        let mut builder = bundle::TransactionBuilder::default();
        builder.
            tag(rec.get::<String, _>(TRANSACTION_COL_TAG).into()).
            address(rec.get::<String, _>(TRANSACTION_COL_ADDRESS).into()).
            nonce(rec.get::<String, _>(TRANSACTION_COL_NONCE).into()).
            attachment_lbts(bundle::Timestamp(attachment_lbts)).
            attachment_ubts(bundle::Timestamp(attachment_ubts)).
            attachment_ts(bundle::Timestamp(attachment_ts)).
            payload(rec.get::<String, _>(TRANSACTION_COL_SIG_OR_MESSAGE).into()).
            index((bundle::Index(index))).
            last_index((bundle::Index(last_index))).
            timestamp(bundle::Timestamp(timestamp)).
            value(bundle::Value(value)).
            obsolete_tag(rec.get::<String, _>(TRANSACTION_COL_OBSOLETE_TAG).into());

        Ok(builder.build())

    }

    async fn update_transactions_set_solid(
        &self,
        transaction_hashes: HashSet<bundle::Hash>,
    ) -> Result<(), SqlxBackendError> {

        let pool = self.0.connection.connection_pool.as_ref().expect(CONNECTION_NOT_INITIALIZED);
        let mut conn_transaction = pool.begin().await?;

        transaction_hashes.iter().for_each(|hash|{
            let row = sqlx::query(
            UPDATE_SET_SOLID_STATEMENT
        )

            .bind(hash.to_string().as_bytes())
            .fetch_one(&mut conn_transaction);

        });

        Ok(())

    }

    async fn update_transactions_set_snapshot_index(
        &self,
        transaction_hashes: HashSet<bundle::Hash>,
        snapshot_index: u32,
    ) -> Result<(), SqlxBackendError> {

        let pool = self.0.connection.connection_pool.as_ref().expect(CONNECTION_NOT_INITIALIZED);
        let mut conn_transaction = pool.begin().await?;

        transaction_hashes.iter().for_each( |hash|{        sqlx::query(
            UPDATE_SNAPSHOT_INDEX_STATEMENT
        )
            .bind(hash.to_string().as_bytes()).bind(snapshot_index as i32)
            .fetch_one(&mut conn_transaction);

        });

        conn_transaction.commit().await?;


        Ok(())
    }

    async fn delete_transactions(&self, transaction_hashes: HashSet<bundle::Hash>) -> Result<(), SqlxBackendError>{

        let pool = self.0.connection.connection_pool.as_ref().expect(CONNECTION_NOT_INITIALIZED);
        let mut conn_transaction = pool.begin().await?;

        transaction_hashes.iter().for_each(|hash|{
            let row = sqlx::query(
                DELETE_TRANSACTION_STATEMENT
            )

                .bind(hash.to_string().as_bytes())
                .fetch_one(&mut conn_transaction);

        });

        conn_transaction.commit().await?;

        Ok(())

    }

    async fn insert_milestone(&self, milestone: &Milestone) -> Result<(), SqlxBackendError>{
        let pool = self.0.connection.connection_pool.as_ref().expect(CONNECTION_NOT_INITIALIZED);
        let mut conn_transaction = pool.begin().await?;


        let row = sqlx::query(
            INSERT_MILESTONE_STATEMENT
        )
            .bind(milestone.index as i32).bind(&milestone.hash.to_string().as_bytes())
            .fetch_one(&mut conn_transaction)
            .await?;

        conn_transaction.commit().await?;

        Ok(())
    }

    async fn find_milestone(&self, milestone_hash: &bundle::Hash) -> Result<Milestone, SqlxBackendError>{

        let mut pool = self.0.connection.connection_pool.as_ref().expect(CONNECTION_NOT_INITIALIZED);

        let user_id : i32 = 0;
        let rec = sqlx::query(
            FIND_MILESTONE_BY_HASH_STATEMENT
        ).bind(milestone_hash.to_string().as_bytes())
            .fetch_one(&mut pool)
            .await?;

        let milestone = Milestone {
            hash: rec.get::<String,_>(MILESTONE_COL_HASH).into(),
            index: rec.get::<i32,_>(MILESTONE_COL_ID) as u32,
        };

        Ok(milestone)

    }

    async fn delete_milestones(
        &self,
        milestone_hashes: HashSet<&bundle::Hash>,
    ) -> Result<(), SqlxBackendError>{

        let pool = self.0.connection.connection_pool.as_ref().expect(CONNECTION_NOT_INITIALIZED);
        let mut conn_transaction = pool.begin().await?;

        milestone_hashes.iter().for_each(|hash|{
            let row = sqlx::query(
                DELETE_MILESTONE_BY_HASH_STATEMENT
            )

                .bind(hash.to_string().as_bytes())
                .fetch_one(&mut conn_transaction);

        });

        conn_transaction.commit().await?;

        Ok(())
    }

    async fn insert_state_delta(
        &self,
        state_delta: storage::StateDeltaMap,
        index: u32,
    ) -> Result<(), SqlxBackendError> {
        let pool = self.0.connection.connection_pool.as_ref().expect(CONNECTION_NOT_INITIALIZED);
        let mut conn_transaction = pool.begin().await?;

        let encoded: Vec<u8> = bincode::serialize(&state_delta)?;

        let row = sqlx::query(
            STORE_DELTA_STATEMENT
        )
            .bind(encoded).bind(index as i32)
            .fetch_one(&mut conn_transaction)
            .await?;

        conn_transaction.commit().await?;

        Ok(())
    }

    async fn load_state_delta(&self, index: u32) -> Result<storage::StateDeltaMap, SqlxBackendError> {

    let mut pool = self.0.connection.connection_pool.as_ref().expect(CONNECTION_NOT_INITIALIZED);

    let rec = sqlx::query(
        LOAD_DELTA_STATEMENT_BY_INDEX
    ).bind(index as i32)
    .fetch_one(&mut pool)
    .await?;

    let delta = rec.get::<String, _>(MILESTONE_COL_DELTA);
    let decoded: storage::StateDeltaMap = bincode::deserialize(&delta.as_bytes())?;

    Ok(decoded)

    }
}

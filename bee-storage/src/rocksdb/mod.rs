extern crate rocksdb;

mod errors;
mod test;

use crate::storage::{
    Connection,
    HashesToApprovers,
    MissingHashesToRCApprovers,
    StateDeltaMap,
    Storage,
    StorageBackend,
};

use bee_protocol::{
    Milestone,
    MilestoneIndex,
};

use bee_bundle::{
    Hash,
    Transaction,
    TransactionField,
};

use bee_ternary::{
    T1B1Buf,
    T5B1Buf,
    TritBuf,
    Trits,
    T5B1,
};

use std::{
    collections::{
        HashMap,
        HashSet,
    },
    mem,
    ptr,
    rc::Rc,
};

use errors::RocksDbBackendError;

use async_trait::async_trait;

use bytemuck::{
    cast_slice,
    cast_slice_mut,
};
use rocksdb::{
    ColumnFamilyDescriptor,
    DBCompactionStyle,
    DBCompressionType,
    IteratorMode,
    Options,
    WriteOptions,
    DB,
};

const TRANSACTION_CF_HASH_TO_TRANSACTION: &str = "transaction_hash_to_transaction";
const TRANSACTION_CF_HASH_TO_SOLID: &str = "transaction_hash_to_solid";
const TRANSACTION_CF_HASH_TO_SNAPSHOT_INDEX: &str = "transaction_hash_to_snapshot_index";
const TRANSACTION_CF_HASH_TO_TRUNK: &str = "transaction_hash_to_trunk";
const TRANSACTION_CF_HASH_TO_BRANCH: &str = "transaction_hash_to_branch";
const MILESTONE_CF_HASH_TO_INDEX: &str = "milestone_hash_to_index";
const MILESTONE_CF_INDEX_TO_HASH: &str = "milestone_index_to_hash";
const MILESTONE_CF_HASH_TO_DELTA: &str = "milestone_hash_to_delta";

#[inline]
fn decode_transaction(buff: &[u8]) -> Transaction {
    let trits =
        unsafe { Trits::<T5B1>::from_raw_unchecked(&cast_slice(buff), Transaction::trits_len()) }.encode::<T1B1Buf>();
    Transaction::from_trits(&trits).unwrap()
}

#[inline]
fn decode_hash(buff: &[u8]) -> Hash {
    let mut hash = Hash::zeros();
    let trits = unsafe { Trits::<T5B1>::from_raw_unchecked(&cast_slice(buff), Hash::trit_len()) }.encode::<T1B1Buf>();
    unsafe {
        ptr::copy(
            trits.as_i8_slice().as_ptr(),
            cast_slice_mut(hash.0.as_mut()).as_mut_ptr(),
            Hash::trit_len(),
        )
    };

    hash
}

pub struct RocksDBBackendConnection {
    db: Option<DB>,
}

impl RocksDBBackendConnection {
    pub fn new() -> Self {
        Self { db: None }
    }
}

#[async_trait]
impl Connection<RocksDBBackendConnection> for RocksDBBackendConnection {
    type StorageError = RocksDbBackendError;

    async fn establish_connection(&mut self, url: &str) -> Result<(), RocksDbBackendError> {
        let mut cf_opts = Options::default();
        cf_opts.set_max_write_buffer_number(16);
        let transaction_cf_hash_to_trnsaction =
            ColumnFamilyDescriptor::new(TRANSACTION_CF_HASH_TO_TRANSACTION, Options::default());
        let transaction_cf_hash_to_solid =
            ColumnFamilyDescriptor::new(TRANSACTION_CF_HASH_TO_SOLID, Options::default());
        let transaction_cf_hash_to_snapshot_index =
            ColumnFamilyDescriptor::new(TRANSACTION_CF_HASH_TO_SNAPSHOT_INDEX, Options::default());

        let transaction_cf_hash_to_trunk =
            ColumnFamilyDescriptor::new(TRANSACTION_CF_HASH_TO_TRUNK, Options::default());

        let transaction_cf_hash_to_branch =
            ColumnFamilyDescriptor::new(TRANSACTION_CF_HASH_TO_BRANCH, Options::default());

        let milestone_cf_hash_to_index = ColumnFamilyDescriptor::new(MILESTONE_CF_HASH_TO_INDEX, Options::default());
        let milestone_cf_index_to_hash = ColumnFamilyDescriptor::new(MILESTONE_CF_INDEX_TO_HASH, Options::default());
        let milestone_cf_hash_to_delta = ColumnFamilyDescriptor::new(MILESTONE_CF_HASH_TO_DELTA, Options::default());
        let mut opts = Options::default();
        //TODO - figure this out
        opts.create_missing_column_families(true);
        opts.create_if_missing(true);
        opts.set_compaction_style(DBCompactionStyle::Universal);
        opts.set_max_background_compactions(4);
        opts.set_max_background_flushes(4);
        opts.set_disable_auto_compactions(true);
        opts.increase_parallelism(num_cpus::get() as i32);
        opts.set_compression_type(DBCompressionType::Zlib);

        self.db = Some(
            DB::open_cf_descriptors(
                &opts,
                url,
                vec![
                    transaction_cf_hash_to_trnsaction,
                    transaction_cf_hash_to_solid,
                    transaction_cf_hash_to_trunk,
                    transaction_cf_hash_to_branch,
                    transaction_cf_hash_to_snapshot_index,
                    milestone_cf_hash_to_index,
                    milestone_cf_index_to_hash,
                    milestone_cf_hash_to_delta,
                ],
            )
            .unwrap(),
        );

        Ok(())
    }
    async fn destroy_connection(&mut self) -> Result<(), RocksDbBackendError> {
        if self.db.is_some() {
            self.db.as_ref().unwrap().flush()?;
        }
        Ok(())
    }
}

pub struct RocksDbBackendStorage(Storage<RocksDBBackendConnection>);

#[async_trait]
impl StorageBackend for RocksDbBackendStorage {
    type StorageError = RocksDbBackendError;

    fn new() -> Self {
        let stor = Storage {
            connection: RocksDBBackendConnection::new(),
        };
        RocksDbBackendStorage(stor)
    }

    async fn establish_connection(&mut self, url: &str) -> Result<(), RocksDbBackendError> {
        let _res = self.0.connection.establish_connection(url).await?;
        Ok(())
    }
    async fn destroy_connection(&mut self) -> Result<(), RocksDbBackendError> {
        let _res = self.0.connection.destroy_connection().await?;
        Ok(())
    }

    fn map_existing_transaction_hashes_to_approvers(&self) -> Result<HashesToApprovers, RocksDbBackendError> {
        let db = self.0.connection.db.as_ref().unwrap();

        let mut hash_to_approvers = HashMap::new();

        let transaction_cf_hash_to_trunk = db.cf_handle(TRANSACTION_CF_HASH_TO_TRUNK).unwrap();
        let transaction_cf_hash_to_branch = db.cf_handle(TRANSACTION_CF_HASH_TO_BRANCH).unwrap();

        for (key, value) in db
            .iterator_cf(&transaction_cf_hash_to_trunk, IteratorMode::Start)
            .unwrap()
            .chain(
                db.iterator_cf(&transaction_cf_hash_to_branch, IteratorMode::Start)
                    .unwrap(),
            )
        {
            let approvee = decode_hash(value.as_ref());
            let approver = decode_hash(key.as_ref());
            hash_to_approvers
                .entry(approvee)
                .or_insert(HashSet::new())
                .insert(approver);
        }

        Ok(hash_to_approvers)
    }

    fn map_missing_transaction_hashes_to_approvers(
        &self,
        all_hashes: HashSet<bee_bundle::Hash>,
    ) -> Result<MissingHashesToRCApprovers, RocksDbBackendError> {
        let db = self.0.connection.db.as_ref().unwrap();

        let mut missing_to_approvers = HashMap::new();
        let transaction_cf_hash_to_trunk = db.cf_handle(TRANSACTION_CF_HASH_TO_TRUNK).unwrap();
        let transaction_cf_hash_to_branch = db.cf_handle(TRANSACTION_CF_HASH_TO_BRANCH).unwrap();
        for (key, value) in db
            .iterator_cf(&transaction_cf_hash_to_trunk, IteratorMode::Start)
            .unwrap()
            .chain(
                db.iterator_cf(&transaction_cf_hash_to_branch, IteratorMode::Start)
                    .unwrap(),
            )
        {
            let approvee = decode_hash(value.as_ref());
            let approver = decode_hash(key.as_ref());

            if !all_hashes.contains(&approvee) {
                let optional_approver_rc = Some(Rc::<bee_bundle::Hash>::new(approver));
                missing_to_approvers
                    .entry(approvee)
                    .or_insert(HashSet::new())
                    .insert(optional_approver_rc.clone().unwrap());
            }
        }

        Ok(missing_to_approvers)
    }
    //Implement all methods here
    async fn insert_transaction(
        &self,
        tx_hash: bee_bundle::Hash,
        tx: bee_bundle::Transaction,
    ) -> Result<(), RocksDbBackendError> {
        let db = self.0.connection.db.as_ref().unwrap();

        let mut tx_trit_buf = TritBuf::<T1B1Buf>::zeros(Transaction::trits_len());

        tx.into_trits_allocated(tx_trit_buf.as_slice_mut());
        let transaction_cf_hash_to_transaction = db.cf_handle(TRANSACTION_CF_HASH_TO_TRANSACTION).unwrap();
        let transaction_cf_hash_to_trunk = db.cf_handle(TRANSACTION_CF_HASH_TO_TRUNK).unwrap();
        let transaction_cf_hash_to_branch = db.cf_handle(TRANSACTION_CF_HASH_TO_BRANCH).unwrap();

        let hash_buf = tx_hash.to_inner().encode::<T5B1Buf>();
        db.put_cf(
            &transaction_cf_hash_to_transaction,
            cast_slice(hash_buf.as_i8_slice()),
            cast_slice(tx_trit_buf.encode::<T5B1Buf>().as_i8_slice()),
        )?;

        db.put_cf(
            &transaction_cf_hash_to_trunk,
            cast_slice(hash_buf.as_i8_slice()),
            cast_slice(tx.trunk().to_inner().encode::<T5B1Buf>().as_i8_slice()),
        )?;

        db.put_cf(
            &transaction_cf_hash_to_branch,
            cast_slice(hash_buf.as_i8_slice()),
            cast_slice(tx.branch().to_inner().encode::<T5B1Buf>().as_i8_slice()),
        )?;

        Ok(())
    }

    async fn insert_transactions(
        &self,
        transactions: HashMap<bee_bundle::Hash, bee_bundle::Transaction>,
    ) -> Result<(), Self::StorageError> {
        let db = self.0.connection.db.as_ref().unwrap();
        let mut batch = rocksdb::WriteBatch::default();
        let transaction_cf_hash_to_transaction = db.cf_handle(TRANSACTION_CF_HASH_TO_TRANSACTION).unwrap();
        let transaction_cf_hash_to_trunk = db.cf_handle(TRANSACTION_CF_HASH_TO_TRUNK).unwrap();
        let transaction_cf_hash_to_branch = db.cf_handle(TRANSACTION_CF_HASH_TO_BRANCH).unwrap();

        let mut tx_trit_buf = TritBuf::<T1B1Buf>::zeros(Transaction::trits_len());
        for (tx_hash, tx) in transactions {
            tx.into_trits_allocated(tx_trit_buf.as_slice_mut());
            let hash_buf = tx_hash.to_inner().encode::<T5B1Buf>();
            batch.put_cf(
                &transaction_cf_hash_to_transaction,
                cast_slice(hash_buf.as_i8_slice()),
                cast_slice(tx_trit_buf.encode::<T5B1Buf>().as_i8_slice()),
            )?;

            batch.put_cf(
                &transaction_cf_hash_to_trunk,
                cast_slice(hash_buf.as_i8_slice()),
                cast_slice(tx.trunk().to_inner().encode::<T5B1Buf>().as_i8_slice()),
            )?;

            batch.put_cf(
                &transaction_cf_hash_to_branch,
                cast_slice(hash_buf.as_i8_slice()),
                cast_slice(tx.branch().to_inner().encode::<T5B1Buf>().as_i8_slice()),
            )?;
        }

        let mut write_options = WriteOptions::default();
        write_options.set_sync(false);
        write_options.disable_wal(true);

        db.write_opt(batch, &write_options)?;
        Ok(())
    }

    async fn find_transaction(
        &self,
        tx_hash: bee_bundle::Hash,
    ) -> Result<bee_bundle::Transaction, RocksDbBackendError> {
        let db = self.0.connection.db.as_ref().unwrap();
        let transaction_cf_hash_to_transaction = db.cf_handle(TRANSACTION_CF_HASH_TO_TRANSACTION).unwrap();
        let res = db.get_cf(
            &transaction_cf_hash_to_transaction,
            cast_slice(tx_hash.to_inner().encode::<T5B1Buf>().as_i8_slice()),
        )?;

        if res.is_none() {
            return Err(RocksDbBackendError::TransactionDoesNotExist);
        }

        Ok(decode_transaction(&res.unwrap()))
    }

    async fn update_transactions_set_solid(
        &self,
        transaction_hashes: HashSet<bee_bundle::Hash>,
    ) -> Result<(), RocksDbBackendError> {
        let db = self.0.connection.db.as_ref().unwrap();
        let mut batch = rocksdb::WriteBatch::default();
        let transaction_cf_hash_to_solid = db.cf_handle(TRANSACTION_CF_HASH_TO_SOLID).unwrap();
        for hash in transaction_hashes {
            let hash_buf = hash.to_inner().encode::<T5B1Buf>();
            batch.put_cf(
                &transaction_cf_hash_to_solid,
                cast_slice(hash_buf.as_i8_slice()),
                unsafe { mem::transmute::<bool, [u8; 1]>(true) },
            )?;
        }

        let mut write_options = WriteOptions::default();
        write_options.set_sync(false);
        write_options.disable_wal(true);

        db.write_opt(batch, &write_options)?;

        Ok(())
    }

    async fn update_transactions_set_snapshot_index(
        &self,
        transaction_hashes: HashSet<bee_bundle::Hash>,
        snapshot_index: MilestoneIndex,
    ) -> Result<(), RocksDbBackendError> {
        let db = self.0.connection.db.as_ref().unwrap();
        let mut batch = rocksdb::WriteBatch::default();
        let transaction_cf_hash_to_snapshot_index = db.cf_handle(TRANSACTION_CF_HASH_TO_SNAPSHOT_INDEX).unwrap();
        for hash in transaction_hashes {
            let hash_buf = hash.to_inner().encode::<T5B1Buf>();
            batch.put_cf(
                &transaction_cf_hash_to_snapshot_index,
                cast_slice(hash_buf.as_i8_slice()),
                snapshot_index.to_le_bytes(),
            )?;
        }

        let mut write_options = WriteOptions::default();
        write_options.set_sync(false);
        write_options.disable_wal(true);

        db.write_opt(batch, &write_options)?;

        Ok(())
    }

    async fn get_transactions_solid_state(
        &self,
        transaction_hashes: Vec<Hash>,
    ) -> Result<Vec<bool>, Self::StorageError> {
        let mut solid_states = vec![false; transaction_hashes.len()];
        let db = self.0.connection.db.as_ref().unwrap();
        let transaction_cf_hash_to_solid = db.cf_handle(TRANSACTION_CF_HASH_TO_SOLID).unwrap();

        for (index, hash) in transaction_hashes.iter().enumerate() {
            if db
                .get_cf(
                    &transaction_cf_hash_to_solid,
                    cast_slice(hash.to_inner().encode::<T5B1Buf>().as_i8_slice()),
                )?
                .is_some()
            {
                //We assume the presence of a value means the transaction is solid
                solid_states[index] = true;
            }
        }

        Ok(solid_states)
    }

    async fn get_transactions_snapshot_index(
        &self,
        transaction_hashes: Vec<Hash>,
    ) -> Result<Vec<u32>, Self::StorageError> {
        let mut solid_states = vec![0 as u32; transaction_hashes.len()];
        let db = self.0.connection.db.as_ref().unwrap();
        let transaction_cf_hash_to_snapshot_index = db.cf_handle(TRANSACTION_CF_HASH_TO_SNAPSHOT_INDEX).unwrap();
        let mut u32_buffer: [u8; 4] = [0, 0, 0, 0];

        for (index, hash) in transaction_hashes.iter().enumerate() {
            let res = db.get_cf(
                &transaction_cf_hash_to_snapshot_index,
                cast_slice(hash.to_inner().encode::<T5B1Buf>().as_i8_slice()),
            )?;
            if res.is_some() {
                //We assume the absence of a value means the transaction is not known to be confirmed
                let transaction_snapshot_index_buffer = res.unwrap();
                unsafe { ptr::copy(transaction_snapshot_index_buffer.as_ptr(), u32_buffer.as_mut_ptr(), 4) };
                solid_states[index] = u32::from_le_bytes(u32_buffer);
            }
        }

        Ok(solid_states)
    }

    async fn delete_transactions(
        &self,
        transaction_hashes: &HashSet<bee_bundle::Hash>,
    ) -> Result<(), RocksDbBackendError> {
        let db = self.0.connection.db.as_ref().unwrap();
        let mut batch = rocksdb::WriteBatch::default();
        let transaction_cf_hash_to_transaction = db.cf_handle(TRANSACTION_CF_HASH_TO_TRANSACTION).unwrap();

        for hash in transaction_hashes {
            let hash_buf = hash.to_inner().encode::<T5B1Buf>();
            batch.delete_cf(&transaction_cf_hash_to_transaction, cast_slice(hash_buf.as_i8_slice()))?;
        }

        let mut write_options = WriteOptions::default();
        write_options.set_sync(false);
        write_options.disable_wal(true);

        db.write_opt(batch, &write_options)?;
        Ok(())
    }

    async fn insert_milestone(&self, milestone: Milestone) -> Result<(), RocksDbBackendError> {
        let db = self.0.connection.db.as_ref().unwrap();

        let milestone_cf_hash_to_index = db.cf_handle(MILESTONE_CF_HASH_TO_INDEX).unwrap();
        let milestone_cf_index_to_hash = db.cf_handle(MILESTONE_CF_INDEX_TO_HASH).unwrap();

        let hash_buf = milestone.hash().to_inner().encode::<T5B1Buf>();
        db.put_cf(
            &milestone_cf_hash_to_index,
            cast_slice(hash_buf.as_i8_slice()),
            milestone.index().to_le_bytes(),
        )?;

        db.put_cf(
            &milestone_cf_index_to_hash,
            milestone.index().to_le_bytes(),
            cast_slice(hash_buf.as_i8_slice()),
        )?;
        Ok(())
    }

    async fn find_milestone(&self, milestone_hash: bee_bundle::Hash) -> Result<Milestone, RocksDbBackendError> {
        let db = self.0.connection.db.as_ref().unwrap();
        let milestone_cf_hash_to_index = db.cf_handle(MILESTONE_CF_HASH_TO_INDEX).unwrap();
        let res = db.get_cf(
            &milestone_cf_hash_to_index,
            cast_slice(milestone_hash.to_inner().encode::<T5B1Buf>().as_i8_slice()),
        )?;

        if res.is_none() {
            return Err(RocksDbBackendError::TransactionDoesNotExist);
        }

        let mut index_buf: [u8; 4] = [0; 4];
        unsafe { ptr::copy(res.unwrap().as_slice().as_ptr(), index_buf.as_mut_ptr(), 4) };
        Ok(Milestone::new(milestone_hash, u32::from_le_bytes(index_buf)))
    }

    async fn delete_milestones(&self, milestone_hashes: &HashSet<bee_bundle::Hash>) -> Result<(), RocksDbBackendError> {
        let db = self.0.connection.db.as_ref().unwrap();

        let milestone_cf_hash_to_index = db.cf_handle(MILESTONE_CF_HASH_TO_INDEX).unwrap();
        let mut batch = rocksdb::WriteBatch::default();

        for hash in milestone_hashes {
            let hash_buf = hash.to_inner().encode::<T5B1Buf>();
            batch.delete_cf(&milestone_cf_hash_to_index, cast_slice(hash_buf.as_i8_slice()))?;
        }

        let mut write_options = WriteOptions::default();
        write_options.set_sync(false);
        write_options.disable_wal(true);

        db.write_opt(batch, &write_options)?;

        Ok(())
    }

    async fn insert_state_delta(
        &self,
        state_delta: StateDeltaMap,
        index: MilestoneIndex,
    ) -> Result<(), RocksDbBackendError> {
        let db = self.0.connection.db.as_ref().unwrap();
        let milestone_cf_hash_to_delta = db.cf_handle(MILESTONE_CF_HASH_TO_DELTA).unwrap();
        //TODO - handle error, assert the milestone exists?
        let encoded: Vec<u8> = bincode::serialize(&state_delta).unwrap();

        db.put_cf(&milestone_cf_hash_to_delta, index.to_le_bytes(), encoded)?;
        Ok(())
    }

    async fn load_state_delta(&self, index: MilestoneIndex) -> Result<StateDeltaMap, RocksDbBackendError> {
        let db = self.0.connection.db.as_ref().unwrap();
        let milestone_cf_hash_to_delta = db.cf_handle(MILESTONE_CF_HASH_TO_DELTA).unwrap();

        let res = db.get_cf(&milestone_cf_hash_to_delta, index.to_le_bytes())?;

        Ok(bincode::deserialize(&res.unwrap()).unwrap())
    }
}

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
    Address,
    Hash,
    Index,
    Nonce,
    Payload,
    Tag,
    Timestamp,
    Transaction,
    TransactionField,
    Value,
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
    TRANSACTION_BYTE_LEN,
    TRANSACTION_TRIT_LEN,
    TRANSACTION_TRYT_LEN,
    TRUNK,
};

use bee_ternary::{
    T1B1Buf,
    T3B1Buf,
    T5B1Buf,
    TritBuf,
    Trits,
    T1B1,
    T3B1,
    T5B1,
};

use std::collections::{
    HashMap,
    HashSet,
};

use std::{
    io::{
        stdout,
        Write,
    },
    mem,
    ptr,
    rc::Rc,
    slice,
    sync::{
        atomic::{
            AtomicPtr,
            AtomicUsize,
            Ordering,
        },
        Arc,
    },
    time::Instant,
};

use std::io;

use serde::{
    Deserialize,
    Serialize,
};

use errors::RocksDbBackendError;

use async_trait::async_trait;
use futures::executor::block_on;

use self::rocksdb::{
    ColumnFamily,
    DBCompactionStyle,
};
use bytemuck::{
    cast_slice,
    cast_slice_mut,
};
use rocksdb::{
    ColumnFamilyDescriptor,
    DBCompactionStyle::Universal,
    DBCompressionType,
    Error,
    IteratorMode,
    Options,
    WriteOptions,
    DB,
};

const TRANSACTION_HASH_COLUMN_FAMILY: &str = "transaction_hash";
const TRANSACTION_HASH_TO_SOLID_COLUMN_FAMILY: &str = "transaction_hash_to_solid";
const TRANSACTION_HASH_TO_SNAPSHOT_INDEX_COLUMN_FAMILY: &str = "transaction_hash_to_snapshot_index";
const TRANSACTION_HASH_TO_TRUNK_COLUMN_FAMILY: &str = "transaction_hash_to_trunk";
const TRANSACTION_HASH_TO_BRANCH_COLUMN_FAMILY: &str = "transaction_hash_to_branch";
const MILESTONE_HASH_COLUMN_FAMILY: &str = "milestone_hash";
const MILESTONE_INDEX_COLUMN_FAMILY: &str = "milestone_index";
const STATE_DELTA_COLUMN_FAMILY: &str = "milestone_hash";

#[inline]
fn decode_transaction(buff: &[u8]) -> Transaction {
    let trits =
        unsafe { Trits::<T5B1>::from_raw_unchecked(&cast_slice(buff), TRANSACTION_TRIT_LEN) }.encode::<T1B1Buf>();
    Transaction::from_trits(&trits.to_owned()).unwrap()
}

#[inline]
fn decode_hash(buff: &[u8]) -> Hash {
    let mut hash = Hash::zeros();
    let trits = unsafe { Trits::<T5B1>::from_raw_unchecked(&cast_slice(buff), HASH_TRIT_LEN) }.to_buf::<T1B1Buf>();
    unsafe {
        ptr::copy(
            trits.as_i8_slice().as_ptr(),
            cast_slice_mut(hash.0.as_mut()).as_mut_ptr(),
            HASH_TRIT_LEN,
        )
    };

    hash
}

#[inline]
fn encode_transaction(tx: &Transaction, mut buf: &mut Trits<T1B1>) {
    //TODO - Index, Value and Timestamp
    unsafe {
        ptr::copy(
            tx.payload().to_inner().as_i8_slice().as_ptr(),
            buf.as_i8_slice_mut()
                .as_mut_ptr()
                .offset(PAYLOAD.trit_offset.start as isize),
            PAYLOAD.trit_offset.length,
        );

        ptr::copy(
            tx.address().to_inner().as_i8_slice().as_ptr(),
            buf.as_i8_slice_mut()
                .as_mut_ptr()
                .offset(ADDRESS.trit_offset.start as isize),
            ADDRESS.trit_offset.length,
        );

        ptr::copy(
            tx.obsolete_tag().to_inner().as_i8_slice().as_ptr(),
            buf.as_i8_slice_mut()
                .as_mut_ptr()
                .offset(OBSOLETE_TAG.trit_offset.start as isize),
            OBSOLETE_TAG.trit_offset.length,
        );

        ptr::copy(
            tx.bundle().to_inner().as_i8_slice().as_ptr(),
            buf.as_i8_slice_mut()
                .as_mut_ptr()
                .offset(BUNDLE.trit_offset.start as isize),
            BUNDLE.trit_offset.length,
        );

        ptr::copy(
            tx.branch().to_inner().as_i8_slice().as_ptr(),
            buf.as_i8_slice_mut()
                .as_mut_ptr()
                .offset(BRANCH.trit_offset.start as isize),
            BRANCH.trit_offset.length,
        );

        ptr::copy(
            tx.trunk().to_inner().as_i8_slice().as_ptr(),
            buf.as_i8_slice_mut()
                .as_mut_ptr()
                .offset(TRUNK.trit_offset.start as isize),
            TRUNK.trit_offset.length,
        );

        ptr::copy(
            tx.tag().to_inner().as_i8_slice().as_ptr(),
            buf.as_i8_slice_mut()
                .as_mut_ptr()
                .offset(TAG.trit_offset.start as isize),
            TAG.trit_offset.length,
        );

        ptr::copy(
            tx.nonce().to_inner().as_i8_slice().as_ptr(),
            buf.as_i8_slice_mut()
                .as_mut_ptr()
                .offset(NONCE.trit_offset.start as isize),
            NONCE.trit_offset.length,
        );
    }
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
        let cf_transaction = ColumnFamilyDescriptor::new(TRANSACTION_HASH_COLUMN_FAMILY, Options::default());
        let cf_transaction_solid =
            ColumnFamilyDescriptor::new(TRANSACTION_HASH_TO_SOLID_COLUMN_FAMILY, Options::default());
        let cf_transaction_snapshot_index =
            ColumnFamilyDescriptor::new(TRANSACTION_HASH_TO_SOLID_COLUMN_FAMILY, Options::default());

        let cf_transaction_trunk =
            ColumnFamilyDescriptor::new(TRANSACTION_HASH_TO_TRUNK_COLUMN_FAMILY, Options::default());

        let cf_transaction_branch =
            ColumnFamilyDescriptor::new(TRANSACTION_HASH_TO_BRANCH_COLUMN_FAMILY, Options::default());

        let cf_milestone_hash = ColumnFamilyDescriptor::new(MILESTONE_HASH_COLUMN_FAMILY, Options::default());
        let cf_milestone_index = ColumnFamilyDescriptor::new(MILESTONE_INDEX_COLUMN_FAMILY, Options::default());
        let cf_state_delta = ColumnFamilyDescriptor::new(STATE_DELTA_COLUMN_FAMILY, Options::default());
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
                    cf_transaction,
                    cf_transaction_solid,
                    cf_transaction_trunk,
                    cf_transaction_branch,
                    cf_transaction_snapshot_index,
                    cf_milestone_hash,
                    cf_milestone_index,
                    cf_state_delta,
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

        let cf_trunk = db.cf_handle(TRANSACTION_HASH_TO_TRUNK_COLUMN_FAMILY).unwrap();
        let cf_branch = db.cf_handle(TRANSACTION_HASH_TO_BRANCH_COLUMN_FAMILY).unwrap();

        for (key, value) in db
            .iterator_cf(&cf_trunk, IteratorMode::Start)
            .unwrap()
            .chain(db.iterator_cf(&cf_branch, IteratorMode::Start).unwrap())
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
        let cf_trunk = db.cf_handle(TRANSACTION_HASH_TO_TRUNK_COLUMN_FAMILY).unwrap();
        let cf_branch = db.cf_handle(TRANSACTION_HASH_TO_BRANCH_COLUMN_FAMILY).unwrap();
        for (key, value) in db
            .iterator_cf(&cf_trunk, IteratorMode::Start)
            .unwrap()
            .chain(db.iterator_cf(&cf_branch, IteratorMode::Start).unwrap())
        {
            let mut optional_approver_rc = None;

            let approvee = decode_hash(value.as_ref());
            let approver = decode_hash(key.as_ref());

            if !all_hashes.contains(&approvee) {
                optional_approver_rc = Some(Rc::<bee_bundle::Hash>::new(approver));
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
        let raw_tx_bytes: &mut [i8] = &mut [0 as i8; TRANSACTION_TRIT_LEN];
        let tx_trits = unsafe { Trits::<T1B1>::from_raw_unchecked_mut(raw_tx_bytes, TRANSACTION_TRIT_LEN) };
        encode_transaction(&tx, tx_trits);
        let transaction_cf = db.cf_handle(TRANSACTION_HASH_COLUMN_FAMILY).unwrap();
        let transaction_trunk_cf = db.cf_handle(TRANSACTION_HASH_TO_TRUNK_COLUMN_FAMILY).unwrap();
        let transaction_branch_cf = db.cf_handle(TRANSACTION_HASH_TO_BRANCH_COLUMN_FAMILY).unwrap();

        let hash_buf = tx_hash.as_trits().to_buf::<T5B1Buf>();
        db.put_cf(
            &transaction_cf,
            cast_slice(hash_buf.as_i8_slice()),
            cast_slice(tx_trits.encode::<T5B1Buf>().as_i8_slice()),
        )?;

        db.put_cf(
            &transaction_trunk_cf,
            cast_slice(hash_buf.as_i8_slice()),
            cast_slice(tx.trunk().as_trits().encode::<T5B1Buf>().as_i8_slice()),
        )?;

        db.put_cf(
            &transaction_branch_cf,
            cast_slice(hash_buf.as_i8_slice()),
            cast_slice(tx.branch().as_trits().encode::<T5B1Buf>().as_i8_slice()),
        )?;

        Ok(())
    }

    async fn find_transaction(
        &self,
        tx_hash: bee_bundle::Hash,
    ) -> Result<bee_bundle::Transaction, RocksDbBackendError> {
        let db = self.0.connection.db.as_ref().unwrap();
        let cf = db.cf_handle(TRANSACTION_HASH_COLUMN_FAMILY).unwrap();
        let res = db.get_cf(&cf, cast_slice(tx_hash.as_trits().to_buf::<T5B1Buf>().as_i8_slice()))?;

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
        let transaction_solid_cf = db.cf_handle(TRANSACTION_HASH_TO_SOLID_COLUMN_FAMILY).unwrap();
        for hash in transaction_hashes {
            let hash_buf = hash.as_trits().to_buf::<T5B1Buf>();
            batch.put_cf(&transaction_solid_cf, cast_slice(hash_buf.as_i8_slice()), unsafe {
                mem::transmute::<bool, [u8; 1]>(true)
            })?;
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
        let transaction_snapshot_index_cf = db.cf_handle(TRANSACTION_HASH_TO_SNAPSHOT_INDEX_COLUMN_FAMILY).unwrap();
        for hash in transaction_hashes {
            let hash_buf = hash.as_trits().to_buf::<T5B1Buf>();
            batch.put_cf(
                &transaction_snapshot_index_cf,
                cast_slice(hash_buf.as_i8_slice()),
                unsafe { mem::transmute::<u32, [u8; 4]>(snapshot_index) },
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
        let transaction_hash_to_solid_cf = db.cf_handle(TRANSACTION_HASH_TO_SOLID_COLUMN_FAMILY).unwrap();

        for (index, hash) in transaction_hashes.iter().enumerate() {
            if db
                .get_cf(
                    &transaction_hash_to_solid_cf,
                    cast_slice(hash.as_trits().encode::<T5B1Buf>().as_i8_slice()),
                )
                .is_ok()
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
        let transaction_hash_to_snapshot_index_cf =
            db.cf_handle(TRANSACTION_HASH_TO_SNAPSHOT_INDEX_COLUMN_FAMILY).unwrap();
        let mut u32_buffer: [u8; 4] = [0, 0, 0, 0];

        for (index, hash) in transaction_hashes.iter().enumerate() {
            let res = db.get_cf(
                &transaction_hash_to_snapshot_index_cf,
                cast_slice(hash.as_trits().encode::<T5B1Buf>().as_i8_slice()),
            );
            if res.is_ok() {
                //We assume the absence of a value means the transaction is not known to be confirmed
                let transaction_snapshot_index_buffer = res.unwrap().unwrap();
                unsafe { ptr::copy(transaction_snapshot_index_buffer.as_ptr(), u32_buffer.as_mut_ptr(), 4) };
                solid_states[index] = unsafe { mem::transmute::<[u8; 4], u32>(u32_buffer) };
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
        let transaction_cf = db.cf_handle(TRANSACTION_HASH_COLUMN_FAMILY).unwrap();

        for hash in transaction_hashes {
            let hash_buf = hash.as_trits().to_buf::<T5B1Buf>();
            batch.delete_cf(&transaction_cf, cast_slice(hash_buf.as_i8_slice()))?;
        }

        let mut write_options = WriteOptions::default();
        write_options.set_sync(false);
        write_options.disable_wal(true);

        db.write_opt(batch, &write_options)?;
        Ok(())
    }

    async fn insert_transactions(
        &self,
        transactions: HashMap<bee_bundle::Hash, bee_bundle::Transaction>,
    ) -> Result<(), Self::StorageError> {
        let db = self.0.connection.db.as_ref().unwrap();
        let mut batch = rocksdb::WriteBatch::default();
        let transaction_cf = db.cf_handle(TRANSACTION_HASH_COLUMN_FAMILY).unwrap();
        let transaction_trunk_cf = db.cf_handle(TRANSACTION_HASH_TO_TRUNK_COLUMN_FAMILY).unwrap();
        let transaction_branch_cf = db.cf_handle(TRANSACTION_HASH_TO_BRANCH_COLUMN_FAMILY).unwrap();

        let raw_tx_bytes: &mut [i8] = &mut [0 as i8; TRANSACTION_TRIT_LEN];
        let tx_trits = unsafe { Trits::<T1B1>::from_raw_unchecked_mut(raw_tx_bytes, TRANSACTION_TRIT_LEN) };

        for (tx_hash, tx) in transactions {
            encode_transaction(&tx, tx_trits);
            let hash_buf = tx_hash.as_trits().to_buf::<T5B1Buf>();
            batch.put_cf(
                &transaction_cf,
                cast_slice(hash_buf.as_i8_slice()),
                cast_slice(tx_trits.encode::<T5B1Buf>().as_i8_slice()),
            )?;

            batch.put_cf(
                &transaction_trunk_cf,
                cast_slice(hash_buf.as_i8_slice()),
                cast_slice(tx.trunk().as_trits().encode::<T5B1Buf>().as_i8_slice()),
            )?;

            batch.put_cf(
                &transaction_branch_cf,
                cast_slice(hash_buf.as_i8_slice()),
                cast_slice(tx.branch().as_trits().encode::<T5B1Buf>().as_i8_slice()),
            )?;
        }

        let mut write_options = WriteOptions::default();
        write_options.set_sync(false);
        write_options.disable_wal(true);

        db.write_opt(batch, &write_options)?;
        Ok(())
    }

    async fn insert_milestone(&self, milestone: Milestone) -> Result<(), RocksDbBackendError> {
        let db = self.0.connection.db.as_ref().unwrap();

        let milestone_hash_cf = db.cf_handle(MILESTONE_HASH_COLUMN_FAMILY).unwrap();
        let milestone_index_cf = db.cf_handle(MILESTONE_INDEX_COLUMN_FAMILY).unwrap();

        let hash_buf = milestone.hash.as_trits().to_buf::<T5B1Buf>();
        db.put_cf(&milestone_hash_cf, cast_slice(hash_buf.as_i8_slice()), unsafe {
            mem::transmute::<u32, [u8; 4]>(milestone.index)
        })?;

        db.put_cf(
            &milestone_index_cf,
            unsafe { mem::transmute::<u32, [u8; 4]>(milestone.index) },
            cast_slice(hash_buf.as_i8_slice()),
        )?;
        Ok(())
    }

    async fn find_milestone(&self, milestone_hash: bee_bundle::Hash) -> Result<Milestone, RocksDbBackendError> {
        let db = self.0.connection.db.as_ref().unwrap();
        let cf = db.cf_handle(MILESTONE_HASH_COLUMN_FAMILY).unwrap();
        let res = db.get_cf(
            &cf,
            cast_slice(milestone_hash.as_trits().to_buf::<T5B1Buf>().as_i8_slice()),
        )?;

        if res.is_none() {
            return Err(RocksDbBackendError::TransactionDoesNotExist);
        }

        let mut index_buf: [u8; 4] = [0; 4];
        unsafe { ptr::copy(res.unwrap().as_slice().as_ptr(), index_buf.as_mut_ptr(), 4) };
        Ok(Milestone {
            hash: milestone_hash,
            index: unsafe { mem::transmute::<[u8; 4], u32>(index_buf) },
        })
    }

    async fn delete_milestones(&self, milestone_hashes: &HashSet<bee_bundle::Hash>) -> Result<(), RocksDbBackendError> {
        let db = self.0.connection.db.as_ref().unwrap();

        let milestone_hash_cf = db.cf_handle(MILESTONE_HASH_COLUMN_FAMILY).unwrap();
        let mut batch = rocksdb::WriteBatch::default();

        for hash in milestone_hashes {
            let hash_buf = hash.as_trits().to_buf::<T5B1Buf>();
            batch.delete_cf(&milestone_hash_cf, cast_slice(hash_buf.as_i8_slice()))?;
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
        let state_delta_cf = db.cf_handle(STATE_DELTA_COLUMN_FAMILY).unwrap();
        //TODO - handle error, assert the milestone exists?
        let encoded: Vec<u8> = bincode::serialize(&state_delta).unwrap();

        db.put_cf(
            &state_delta_cf,
            unsafe { mem::transmute::<u32, [u8; 4]>(index) },
            encoded,
        )?;
        Ok(())
    }

    async fn load_state_delta(&self, index: MilestoneIndex) -> Result<StateDeltaMap, RocksDbBackendError> {
        let db = self.0.connection.db.as_ref().unwrap();
        let state_delta_cf = db.cf_handle(STATE_DELTA_COLUMN_FAMILY).unwrap();

        let res = db.get_cf(&state_delta_cf, unsafe { mem::transmute::<u32, [u8; 4]>(index) })?;

        Ok(bincode::deserialize(&res.unwrap()).unwrap())
    }
}

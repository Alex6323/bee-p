extern crate serde;

use bee_bundle::{
    Address,
    Hash,
};
use bee_protocol::{
    Milestone,
    MilestoneIndex,
};

use std::{
    collections::{
        HashMap,
        HashSet,
    },
    fmt::Debug,
    rc::Rc,
};

use serde::{
    Deserialize,
    Serialize,
};

use async_trait::async_trait;

// A transaction address. To be replaced later with whatever implementation is required.
//type TxAddress = String;

pub type HashesToApprovers = HashMap<Hash, HashSet<Hash>>;
pub type MissingHashesToRCApprovers = HashMap<Hash, HashSet<Rc<Hash>>>;
//This is a mapping between an iota address and it's balance change
//practically, a map for total balance change over an addresses will be collected
//per milestone (snapshot_index), when we no longer have milestones, we will have to find
//another way to decide on a check point where to store an address's delta if we want to snapshot
#[derive(Serialize, Deserialize, Default, Debug)]
pub struct StateDeltaMap {
    pub address_to_delta: HashMap<Address, i64>,
}

pub struct AttachmentData {
    pub hash: Hash,
    pub trunk: Hash,
    pub branch: Hash,
}

#[async_trait]
pub trait Connection<Conn> {
    type StorageError: Debug;
    async fn establish_connection(&mut self, url: &str) -> Result<(), Self::StorageError>;
    async fn destroy_connection(&mut self) -> Result<(), Self::StorageError>;
}

#[async_trait]
pub trait StorageBackend {
    type StorageError: Debug;

    fn new() -> Self;
    async fn establish_connection(&mut self, url: &str) -> Result<(), Self::StorageError>;
    async fn destroy_connection(&mut self) -> Result<(), Self::StorageError>;
    //This method is heavy weighted and will be used to populate Tangle struct on initialization
    //**Operations over transaction's schema**//
    fn map_existing_transaction_hashes_to_approvers(&self) -> Result<HashesToApprovers, Self::StorageError>;

    //This method is heavy weighted and will be used to populate Tangle struct on initialization
    fn map_missing_transaction_hashes_to_approvers(
        &self,
        all_hashes: HashSet<Hash>,
    ) -> Result<MissingHashesToRCApprovers, Self::StorageError>;

    async fn insert_transaction(&self, tx_hash: Hash, tx: bee_bundle::Transaction) -> Result<(), Self::StorageError>;
    async fn insert_transactions(
        &self,
        transactions: HashMap<Hash, bee_bundle::Transaction>,
    ) -> Result<(), Self::StorageError>;
    async fn find_transaction(&self, tx_hash: Hash) -> Result<bee_bundle::Transaction, Self::StorageError>;
    async fn update_transactions_set_solid(&self, transaction_hashes: HashSet<Hash>) -> Result<(), Self::StorageError>;
    async fn update_transactions_set_snapshot_index(
        &self,
        transaction_hashes: HashSet<Hash>,
        snapshot_index: MilestoneIndex,
    ) -> Result<(), Self::StorageError>;

    async fn get_transactions_solid_state(
        &self,
        transaction_hashes: Vec<Hash>,
    ) -> Result<Vec<bool>, Self::StorageError>;

    async fn get_transactions_snapshot_index(
        &self,
        transaction_hashes: Vec<Hash>,
    ) -> Result<Vec<u32>, Self::StorageError>;

    //**Operations over milestone's schema**//

    async fn delete_transactions(&self, transaction_hashes: &HashSet<Hash>) -> Result<(), Self::StorageError>;

    async fn insert_milestone(&self, milestone: Milestone) -> Result<(), Self::StorageError>;

    async fn find_milestone(&self, milestone_hash: Hash) -> Result<Milestone, Self::StorageError>;

    async fn delete_milestones(&self, milestone_hashes: &HashSet<Hash>) -> Result<(), Self::StorageError>;

    //**Operations over state_delta's schema**//

    async fn insert_state_delta(
        &self,
        state_delta: StateDeltaMap,
        index: MilestoneIndex,
    ) -> Result<(), Self::StorageError>;

    async fn load_state_delta(&self, index: MilestoneIndex) -> Result<StateDeltaMap, Self::StorageError>;
}

#[derive(Clone, Debug)]
pub struct Storage<Conn: Connection<Conn>> {
    pub connection: Conn,
}

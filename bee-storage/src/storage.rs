extern crate serde;

//use iota_lib_rs::iota_model::Transaction;
//use std::fmt::{self, Debug, Display};
//use std::io;

use std::collections::{
    HashMap,
    HashSet,
};
use std::fmt::Debug;
use std::rc::Rc;

use serde::{
    Deserialize,
    Serialize,
};

// A transaction address. To be replaced later with whatever implementation is required.
//type TxAddress = String;

pub type HashesToApprovers = HashMap<bee_bundle::Hash, HashSet<bee_bundle::Hash>>;
pub type MissingHashesToRCApprovers = HashMap<bee_bundle::Hash, HashSet<Rc<bee_bundle::Hash>>>;
//This is a mapping between an iota address and it's balance change
//practically, a map for total balance change over an addresses will be collected
//per milestone (snapshot_index), when we no longer have milestones, we will have to find
//another way to decide on a check point where to store an address's delta if we want to snapshot
#[derive(Serialize, Deserialize, Default, Debug)]
pub struct StateDeltaMap {
    #[serde(flatten)]
    address_to_delta: HashMap<Address, i64>,
}

use async_trait::async_trait;
use bee_bundle::{
    Address,
    Hash,
};

#[async_trait]
pub trait Connection<Conn> {
    type StorageError;
    async fn establish_connection(&mut self, url: &str) -> Result<(), Self::StorageError>;
    async fn destroy_connection(&mut self) -> Result<(), Self::StorageError>;
}

#[async_trait]
pub trait StorageBackend {
    type StorageError;
    //This method is heavy weighted and will be used to populate Tangle struct on initialization
    //**Operations over transaction's schema**//
    fn map_existing_transaction_hashes_to_approvers(&self) -> Result<HashesToApprovers, Self::StorageError>;

    //This method is heavy weighted and will be used to populate Tangle struct on initialization
    fn map_missing_transaction_hashes_to_approvers(
        &self,
        all_hashes: HashSet<bee_bundle::Hash>,
    ) -> Result<MissingHashesToRCApprovers, Self::StorageError>;

    async fn insert_transaction(
        &self,
        tx_hash: bee_bundle::Hash,
        tx: bee_bundle::Transaction,
    ) -> Result<(), Self::StorageError>;
    async fn insert_transactions(
        &self,
        transactions: HashMap<bee_bundle::Hash, bee_bundle::Transaction>,
    ) -> Result<(), Self::StorageError>;
    async fn find_transaction(&self, tx_hash: bee_bundle::Hash) -> Result<bee_bundle::Transaction, Self::StorageError>;
    async fn update_transactions_set_solid(
        &self,
        transaction_hashes: HashSet<bee_bundle::Hash>,
    ) -> Result<(), Self::StorageError>;
    async fn update_transactions_set_snapshot_index(
        &self,
        transaction_hashes: HashSet<bee_bundle::Hash>,
        snapshot_index: u32,
    ) -> Result<(), Self::StorageError>;

    //**Operations over milestone's schema**//

    async fn delete_transactions(
        &self,
        transaction_hashes: &HashSet<bee_bundle::Hash>,
    ) -> Result<(), Self::StorageError>;

    async fn insert_milestone(&self, milestone: bee_bundle::Milestone) -> Result<(), Self::StorageError>;

    async fn find_milestone(
        &self,
        milestone_hash: bee_bundle::Hash,
    ) -> Result<bee_bundle::Milestone, Self::StorageError>;

    async fn delete_milestones(&self, milestone_hashes: &HashSet<bee_bundle::Hash>) -> Result<(), Self::StorageError>;

    //**Operations over state_delta's schema**//

    async fn insert_state_delta(&self, state_delta: StateDeltaMap, index: u32) -> Result<(), Self::StorageError>;

    async fn load_state_delta(&self, index: u32) -> Result<StateDeltaMap, Self::StorageError>;
}

#[derive(Clone, Debug)]
pub struct Storage<Conn: Connection<Conn>> {
    pub connection: Conn,
}

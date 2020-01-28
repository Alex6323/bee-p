extern crate serde;

use iota_lib_rs::iota_model::Transaction;
use std::fmt::{self, Debug, Display};
use std::io;
use std::{collections::HashMap,collections::HashSet, rc::Rc};

pub use bundle::*;

use serde::{Serialize, Deserialize};



/// A transaction address. To be replaced later with whatever implementation is required.
type TxAddress = String;


#[derive(Default, Debug)]
pub struct Milestone {
    pub hash: bundle::Hash,
    pub index: u32,
}

pub type HashesToApprovers = HashMap<bundle::Hash, HashSet<bundle::Hash>>;
pub type MissingHashesToRCApprovers = HashMap<bundle::Hash, HashSet<Rc<bundle::Hash>>>;
//This is a mapping between an iota address and it's balance change
//practically, a map for total balance change over an addresses will be collected
//per milestone (snapshot_index), when we no longer have milestones, we will have to find
//another way to decide on a check point where to store an address's delta if we want to snapshot
#[derive(Serialize, Deserialize, Default, Debug)]
pub struct StateDeltaMap{
    #[serde(flatten)]
    address_to_delta: HashMap<String, i64>,
}

use async_trait::async_trait;

#[async_trait]
pub trait Connection<Conn> {
    type StorageError;
    async fn establish_connection(&mut self) -> Result<(), Self::StorageError>;
    async fn destroy_connection(&mut self) -> Result<(), Self::StorageError>;
}

#[async_trait]
pub trait StorageBackend {

    type StorageError;
    //This method is heavy weighted and will be used to populate Tangle struct on initialization
    //**Operations over transaction's schema**//
    fn map_existing_transaction_hashes_to_approvers(
        &self,
    ) -> Result<HashesToApprovers, Self::StorageError>;

    //This method is heavy weighted and will be used to populate Tangle struct on initialization
    fn map_missing_transaction_hashes_to_approvers(
        &self, all_hashes: HashSet<bundle::Hash>
    ) -> Result<MissingHashesToRCApprovers, Self::StorageError>;

    async fn insert_transaction(&self, tx_hash: &bundle::Hash, tx: &bundle::Transaction) -> Result<(), Self::StorageError>;
    async fn find_transaction(&self, tx_hash: &bundle::Hash) -> Result<bundle::Transaction, Self::StorageError>;
    async fn update_transactions_set_solid(
        &self,
        transaction_hashes: HashSet<bundle::Hash>,
    ) -> Result<(), Self::StorageError>;
    async fn update_transactions_set_snapshot_index(
        &self,
        transaction_hashes: HashSet<bundle::Hash>,
        snapshot_index: u32,
    ) -> Result<(), Self::StorageError>;

    //**Operations over milestone's schema**//

    async fn delete_transactions(&self, transaction_hashes: HashSet<bundle::Hash>) -> Result<(), Self::StorageError>;

    async fn insert_milestone(&self, milestone: &Milestone) -> Result<(), Self::StorageError>;

    async fn find_milestone(&self, milestone_hash: &bundle::Hash) -> Result<Milestone, Self::StorageError>;

    async fn delete_milestones(
        &self,
        milestone_hashes: HashSet<&bundle::Hash>,
    ) -> Result<(), Self::StorageError>;


    //**Operations over state_delta's schema**//

    async fn insert_state_delta(
        &self,
        state_delta: StateDeltaMap,
        index: u32,
    ) -> Result<(), Self::StorageError>;

    async fn load_state_delta(&self, index: u32) -> Result<StateDeltaMap, Self::StorageError>;

}

pub struct Storage<Conn: Connection<Conn>> {
    pub connection: Conn,
}



#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}

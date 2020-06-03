// Copyright 2020 IOTA Stiftung
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file except in compliance with
// the License. You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software distributed under the License is distributed on
// an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and limitations under the License.

use bee_protocol::{Milestone, MilestoneIndex};
use bee_transaction::{Address, BundledTransaction as Transaction, Hash};

use async_trait::async_trait;

use std::{
    collections::{HashMap, HashSet},
    fmt::Debug,
    rc::Rc,
};

// A transaction address. To be replaced later with whatever implementation is required.
// type TxAddress = String;

pub type HashesToApprovers = HashMap<Hash, HashSet<Hash>>;
pub type MissingHashesToRCApprovers = HashMap<Hash, HashSet<Rc<Hash>>>;
// This is a mapping between an iota address and it's balance change
// practically, a map for total balance change over an addresses will be collected
// per milestone (snapshot_index), when we no longer have milestones, we will have to find
// another way to decide on a check point where to store an address's delta if we want to snapshot
#[derive(Default, Debug)]
pub struct StateDeltaMap {
    pub address_to_delta: HashMap<Address, i64>,
}

pub struct AttachmentData {
    pub hash: Hash,
    pub trunk: Hash,
    pub branch: Hash,
}

#[async_trait]
pub trait Connection {
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
    // This method is heavy weighted and will be used to populate Tangle struct on initialization
    //**Operations over transaction's schema**//
    fn map_existing_transaction_hashes_to_approvers(&self) -> Result<HashesToApprovers, Self::StorageError>;

    // This method is heavy weighted and will be used to populate Tangle struct on initialization
    fn map_missing_transaction_hashes_to_approvers(
        &self,
        all_hashes: HashSet<Hash>,
    ) -> Result<MissingHashesToRCApprovers, Self::StorageError>;

    async fn insert_transaction(&self, tx_hash: Hash, tx: Transaction) -> Result<(), Self::StorageError>;
    async fn insert_transactions(&self, transactions: HashMap<Hash, Transaction>) -> Result<(), Self::StorageError>;
    async fn find_transaction(&self, tx_hash: Hash) -> Result<Transaction, Self::StorageError>;
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
pub struct Storage<Conn: Connection> {
    pub(crate) connection: Conn,
}

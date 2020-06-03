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

use crate::{backends::rocksdb::RocksDbBackendError, models::MilestoneStorage};

use bee_protocol::Milestone;
use bee_transaction::Hash;

use async_trait::async_trait;

use std::collections::HashSet;

struct RocksDbMilestoneStorage;

// #[async_trait]
// impl MilestoneStorage for RocksDbMilestoneStorage {
//     type Error = RocksDbBackendError;
//
//     async fn insert(&self, milestone: Milestone) -> Result<(), Self::Error> {}
//
//     async fn get(&self, hash: Hash) -> Result<Milestone, Self::Error> {}
//
//     async fn remove(&self, hash: Hash) -> Result<(), Self::Error> {}
//
//     async fn remove_set(&self, hashes: &HashSet<Hash>) -> Result<(), Self::Error> {}
// }

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

use crate::{access::Error, storage::Backend};

#[async_trait::async_trait]
pub trait BatchBuilder<'a, S: Backend, K, V>: Sized {
    type Error: Error;

    fn try_insert(&mut self, key: &K, value: &V) -> Result<(), Self::Error>;

    fn try_delete(&mut self, key: &K) -> Result<(), Self::Error>;

    fn insert(&mut self, key: &K, value: &V) {
        self.try_insert(key, value).unwrap()
    }

    fn delete(&mut self, key: &K) {
        self.try_delete(key).unwrap()
    }
}

pub trait BeginBatch<'a>: Backend + Sized {
    type BatchBuilder: CommitBatch;

    fn begin_batch(&'a self) -> Self::BatchBuilder;
}

#[async_trait::async_trait]
pub trait CommitBatch {
    type Error: Error;

    async fn commit_batch(self, durability: bool) -> Result<(), Self::Error>;
}

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

use crate::storage::Backend;

#[async_trait::async_trait]
pub trait Batch<K, V>: Sized + Backend {
    type BatchBuilder: Send + Sized;

    fn begin_batch() -> Self::BatchBuilder;

    fn batch_try_insert(&self, batch: &mut Self::BatchBuilder, key: &K, value: &V) -> Result<(), Self::Error>;

    fn batch_try_delete(&self, batch: &mut Self::BatchBuilder, key: &K) -> Result<(), Self::Error>;

    fn batch_insert(&self, batch: &mut Self::BatchBuilder, key: &K, value: &V) {
        self.batch_try_insert(batch, key, value).unwrap()
    }

    fn batch_delete(&self, batch: &mut Self::BatchBuilder, key: &K) {
        self.batch_try_delete(batch, key).unwrap()
    }

    async fn commit_batch(&self, batch: Self::BatchBuilder, durability: bool) -> Result<(), Self::Error>;
}

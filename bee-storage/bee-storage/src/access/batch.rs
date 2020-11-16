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
pub trait BatchBuilder: Backend + Sized {
    type Batch: Default + Send + Sized;

    fn batch_begin() -> Self::Batch {
        Self::Batch::default()
    }

    async fn batch_commit(&self, batch: Self::Batch, durability: bool) -> Result<(), Self::Error>;
}

pub trait Batch<K, V>: Backend + BatchBuilder + Sized {
    fn batch_insert(&self, batch: &mut Self::Batch, key: &K, value: &V) -> Result<(), Self::Error>;

    fn batch_delete(&self, batch: &mut Self::Batch, key: &K) -> Result<(), Self::Error>;
}

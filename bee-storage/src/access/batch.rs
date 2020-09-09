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
pub trait BatchBuilder<'a, S: Backend, K, V>: Sized {
    type Error: std::fmt::Debug;
    fn try_insert(self, key: K, value: V) -> Result<Self, (Self, Self::Error)>;
    fn try_delete(self, key: K) -> Result<Self, (Self, Self::Error)>;
    fn insert(self, key: K, value: V) -> Self {
        self.try_insert(key, value).map_err(|(_, e)| e).unwrap()
    }
    fn delete(self, key: K) -> Self {
        self.try_delete(key).map_err(|(_, e)| e).unwrap()
    }
    fn apply(self, durability: bool) -> Result<(), Self::Error>;
}

pub trait Batch<'a, K, V>: Backend + Sized {
    type BatchBuilder;
    fn create_batch(&'a self) -> Self::BatchBuilder;
}

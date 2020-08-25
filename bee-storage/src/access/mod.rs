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

//! A crate that contains foundational building blocks for the IOTA Tangle.

pub mod ledger_diff;
pub mod milestone;
pub mod transaction;
pub mod transaction_metadata;

pub enum OpError {
    // todo add operations errors
    Unknown(String),
}

#[cfg(feature = "rocks_db")]
impl From<::rocksdb::Error> for OpError {
    fn from(err: ::rocksdb::Error) -> Self {
        OpError::Unknown(err.into_string())
    }
}

#[macro_export]
macro_rules! impl_ops {
    (transaction: $object:ty) => {
        crate::impl_transaction_ops!($object);
    };
    (transaction_metadata: $object:ty) => {
        crate::impl_transaction_metadata_ops!($object);
    };
    (milestone: $object:ty) => {
        crate::impl_milestone_ops!($object);
    };
    (ledger_diff: $object:ty) => {
        crate::impl_ledger_diff_ops!($object);
    };
}

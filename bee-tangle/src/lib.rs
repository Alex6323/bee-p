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

#![warn(missing_docs)]

pub use tangle::Tangle;

pub mod traversal;

mod tangle;
mod vertex;

use bee_transaction::BundledTransaction as Transaction;

use async_std::sync::Arc;

use std::ops::Deref;

/// A thread-safe reference to a `bee_transaction:BundledTransaction`.
#[derive(Clone)]
pub struct TransactionRef(pub(crate) Arc<Transaction>);

impl Deref for TransactionRef {
    type Target = Transaction;

    fn deref(&self) -> &Self::Target {
        &*self.0
    }
}

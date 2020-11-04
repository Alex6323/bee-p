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

use crate::{error::Error, output::Output, spent::Spent, unspent::Unspent};

use bee_message::payload::transaction::OutputId;
use bee_storage::{
    access::{BeginBatch, CommitBatch, Delete, Exist, Fetch, Insert},
    storage,
};

pub trait Backend:
    storage::Backend
    // + for<'a> BeginBatch<'a>
    + Insert<OutputId, Output>
    + Insert<OutputId, Spent>
    + Insert<Unspent, ()>
    + Fetch<OutputId, Output>
    + Fetch<OutputId, Spent>
    + Exist<OutputId, Output>
    + Exist<OutputId, Spent>
    + Exist<Unspent, ()>
    + Delete<OutputId, Output>
    + Delete<OutputId, Spent>
    + Delete<Unspent,()>
// where
//     for<'a> <Self as BeginBatch<'a>>::BatchBuilder: CommitBatch,
{
}

impl<T> Backend for T where
    T: storage::Backend
        + Insert<OutputId, Output>
        + Insert<OutputId, Spent>
        + Insert<Unspent, ()>
        + Fetch<OutputId, Output>
        + Fetch<OutputId, Spent>
        + Exist<OutputId, Output>
        + Exist<OutputId, Spent>
        + Exist<Unspent, ()>
        + Delete<OutputId, Output>
        + Delete<OutputId, Spent>
        + Delete<Unspent, ()> // for<'a> <Self as BeginBatch<'a>>::BatchBuilder: CommitBatch,
{
}

pub(crate) async fn is_output_unspent<B: Backend>(storage: &B, output_id: &OutputId) -> Result<bool, Error> {
    storage
        .exist(&Unspent::new(*output_id))
        .await
        .map_err(|e| Error::Storage(Box::new(e)))
}

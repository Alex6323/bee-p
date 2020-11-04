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

use crate::{output::Output, spent::Spent};

use bee_message::payload::transaction::OutputId;
use bee_storage::{
    access::{Delete, Exist, Fetch, Insert},
    storage,
};

pub trait Backend:
    storage::Backend
    + Insert<OutputId, Output>
    + Insert<OutputId, Spent>
    + Fetch<OutputId, Output>
    + Fetch<OutputId, Spent>
    + Exist<OutputId, Output>
    + Exist<OutputId, Spent>
    + Delete<OutputId, Output>
    + Delete<OutputId, Spent>
{
}

impl<T> Backend for T where
    T: storage::Backend
        + Insert<OutputId, Output>
        + Insert<OutputId, Spent>
        + Fetch<OutputId, Output>
        + Fetch<OutputId, Spent>
        + Exist<OutputId, Output>
        + Exist<OutputId, Spent>
        + Delete<OutputId, Output>
        + Delete<OutputId, Spent>
{
}

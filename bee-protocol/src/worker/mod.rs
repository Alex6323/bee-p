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

mod broadcaster;
mod bundle_validator;
mod milestone_validator;
mod peer;
mod requester;
mod responder;
mod solidifier;
mod status;
mod tps;
mod transaction;

pub(crate) use broadcaster::{BroadcasterWorker, BroadcasterWorkerEvent};
pub(crate) use bundle_validator::{BundleValidatorWorker, BundleValidatorWorkerEvent};
pub(crate) use milestone_validator::MilestoneValidatorWorker;
pub(crate) use peer::{PeerHandshakerWorker, PeerWorker};
pub(crate) use requester::{
    MilestoneRequesterWorker, MilestoneRequesterWorkerEntry, TransactionRequesterWorker,
    TransactionRequesterWorkerEntry,
};
pub(crate) use responder::{
    MilestoneResponderWorker, MilestoneResponderWorkerEvent, TransactionResponderWorker,
    TransactionResponderWorkerEvent,
};
pub(crate) use solidifier::{
    KickstartWorker, MilestoneSolidifierWorker, MilestoneSolidifierWorkerEvent, SolidPropagatorWorker,
    SolidPropagatorWorkerEvent,
};
pub(crate) use status::StatusWorker;
pub(crate) use tps::TpsWorker;
pub(crate) use transaction::{HasherWorker, HasherWorkerEvent, ProcessorWorker};

use bee_common::worker::Error as WorkerError;

use async_trait::async_trait;
use futures::Stream;

use std::any::TypeId;

#[async_trait]
pub(crate) trait Worker {
    const DEPS: &'static [TypeId];

    type Event;
    type Receiver: Stream<Item = Self::Event>;

    async fn run(self, receiver: Self::Receiver) -> Result<(), WorkerError>;
}

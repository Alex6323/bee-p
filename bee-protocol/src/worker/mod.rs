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
mod milestone_validator;
mod peer;
mod requester;
mod responder;
mod sender;
mod solidifier;
mod status;
mod tps;
mod transaction;

pub(crate) use broadcaster::{BroadcasterWorker, BroadcasterWorkerEvent};
pub(crate) use milestone_validator::{MilestoneValidatorWorker, MilestoneValidatorWorkerEvent};
pub(crate) use peer::{PeerHandshakerWorker, PeerWorker};
pub(crate) use requester::{
    MilestoneRequesterWorker, MilestoneRequesterWorkerEntry, TransactionRequesterWorker,
    TransactionRequesterWorkerEntry,
};
pub(crate) use responder::{
    MilestoneResponderWorker, MilestoneResponderWorkerEvent, TransactionResponderWorker,
    TransactionResponderWorkerEvent,
};
pub(crate) use sender::SenderWorker;
pub(crate) use solidifier::{
    MilestoneSolidifierWorker, MilestoneSolidifierWorkerEvent, TransactionSolidifierWorker,
    TransactionSolidifierWorkerEvent,
};
pub(crate) use status::StatusWorker;
pub(crate) use tps::TpsWorker;
pub(crate) use transaction::{TransactionWorker, TransactionWorkerEvent};

use futures::{
    channel::oneshot,
    future::{self, FutureExt},
    stream::{self, Stream, StreamExt},
};

pub(crate) struct Receiver<R> {
    receiver: stream::Fuse<R>,
    shutdown: future::Fuse<oneshot::Receiver<()>>,
}

impl<R, E> Receiver<R>
where
    R: Stream<Item = E> + std::marker::Unpin,
{
    pub(crate) fn new(receiver: R, shutdown: oneshot::Receiver<()>) -> Self {
        Self {
            receiver: receiver.fuse(),
            shutdown: shutdown.fuse(),
        }
    }

    async fn receive_event(&mut self) -> Option<E> {
        loop {
            futures::select! {
                _ = &mut self.shutdown => return None,
                event = self.receiver.next() => if let Some(_) = event {
                    return event;
                }
            }
        }
    }
}

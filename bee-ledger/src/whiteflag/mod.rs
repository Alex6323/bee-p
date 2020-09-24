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

mod b1t6;
mod merkle_hasher;
mod metadata;
mod traversal;
mod worker;

use crate::state::LedgerState;

use worker::LedgerWorker;
pub use worker::LedgerWorkerEvent;

use bee_common::shutdown_stream::ShutdownStream;
use bee_common_ext::{bee_node::BeeNode, event::Bus, worker::Worker};
use bee_protocol::{config::ProtocolCoordinatorConfig, event::LatestSolidMilestoneChanged, MilestoneIndex};

use async_std::task::spawn;
use futures::channel::{mpsc, oneshot};
use log::warn;

use std::sync::Arc;

pub fn init(
    index: u32,
    state: LedgerState,
    coo_config: ProtocolCoordinatorConfig,
    bee_node: Arc<BeeNode>,
    bus: Arc<Bus<'static>>,
) -> mpsc::UnboundedSender<LedgerWorkerEvent> {
    // TODO
    // if unsafe { !WHITE_FLAG.is_null() } {
    //     warn!("Already initialized.");
    //     return;
    // }

    let (ledger_worker_tx, ledger_worker_rx) = mpsc::unbounded();
    let (_, ledger_worker_shutdown_rx) = oneshot::channel();

    spawn(
        LedgerWorker::new(MilestoneIndex(index), state, coo_config, bus.clone()).start(
            ShutdownStream::new(ledger_worker_shutdown_rx, ledger_worker_rx),
            bee_node,
            (),
        ),
    );

    let ledger_worker_tx_ret = ledger_worker_tx.clone();

    bus.add_listener(move |latest_solid_milestone: &LatestSolidMilestoneChanged| {
        if let Err(e) = ledger_worker_tx.unbounded_send(LedgerWorkerEvent::Confirm(latest_solid_milestone.0.clone())) {
            warn!(
                "Sending solid milestone {:?} to confirmation failed: {:?}.",
                latest_solid_milestone.0.index(),
                e
            );
        }
    });

    ledger_worker_tx_ret
}

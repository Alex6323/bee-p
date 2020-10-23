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

//#![warn(missing_docs)]

pub mod event;
mod merkle_hasher;
mod metadata;
mod white_flag;
mod worker;

use worker::LedgerWorker;
pub use worker::LedgerWorkerEvent;

use bee_common_ext::{
    event::Bus,
    node::{Node, NodeBuilder},
};
use bee_protocol::{config::ProtocolCoordinatorConfig, event::LatestSolidMilestoneChanged, MilestoneIndex};

use log::warn;

use std::sync::Arc;

pub fn init<N: Node>(
    index: u32,
    coo_config: ProtocolCoordinatorConfig,
    node_builder: N::Builder,
    bus: Arc<Bus<'static>>,
) -> N::Builder {
    node_builder.with_worker_cfg::<LedgerWorker>((MilestoneIndex(index), coo_config, bus.clone()))
}

pub fn events<N: Node>(node: &N, bus: Arc<Bus<'static>>) {
    let ledger_worker = node.worker::<LedgerWorker>().unwrap().tx.clone();

    bus.add_listener(move |latest_solid_milestone: &LatestSolidMilestoneChanged| {
        if let Err(e) = ledger_worker.send(LedgerWorkerEvent::Confirm(latest_solid_milestone.0.clone())) {
            warn!(
                "Sending solid milestone {:?} to confirmation failed: {:?}.",
                latest_solid_milestone.0.index(),
                e
            );
        }
    });
}

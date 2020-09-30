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

use bee_common_ext::{
    bee_node::BeeNode,
    event::Bus,
    node::{Node, NodeBuilder},
};
use bee_protocol::{config::ProtocolCoordinatorConfig, event::LatestSolidMilestoneChanged, MilestoneIndex};
use bee_storage::storage::Backend;

use log::warn;

use std::sync::Arc;

pub fn init<B: Backend>(
    index: u32,
    state: LedgerState,
    coo_config: ProtocolCoordinatorConfig,
    node_builder: NodeBuilder<BeeNode<B>>,
    bus: Arc<Bus<'static>>,
) -> NodeBuilder<BeeNode<B>> {
    node_builder.with_worker_cfg::<LedgerWorker>((MilestoneIndex(index), state, coo_config, bus.clone()))
}

pub fn events<B: Backend>(bee_node: &BeeNode<B>, bus: Arc<Bus<'static>>) {
    let ledger_worker = bee_node.worker::<LedgerWorker>().unwrap().tx.clone();

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

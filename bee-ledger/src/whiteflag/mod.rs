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

mod confirmation;
mod merkle;
mod traversal;
mod worker;

use worker::LedgerWorker;
pub use worker::LedgerWorkerEvent;

use bee_common::{shutdown::Shutdown, shutdown_stream::ShutdownStream};
use bee_common_ext::event::Bus;
use bee_protocol::{event::LastSolidMilestoneChanged, MilestoneIndex};
use bee_transaction::bundled::Address;

use async_std::task::spawn;
use futures::channel::{mpsc, oneshot};
use log::warn;

use std::{collections::HashMap, ptr, sync::Arc};

struct WhiteFlag {
    pub(crate) bus: Arc<Bus<'static>>,
    confirmation_sender: mpsc::UnboundedSender<LedgerWorkerEvent>,
}

static mut WHITE_FLAG: *const WhiteFlag = ptr::null();

impl WhiteFlag {
    fn get() -> &'static WhiteFlag {
        if unsafe { WHITE_FLAG.is_null() } {
            panic!("Uninitialized whiteflag.");
        } else {
            unsafe { &*WHITE_FLAG }
        }
    }
}

fn on_last_solid_milestone_changed(last_solid_milestone: &LastSolidMilestoneChanged) {
    if let Err(e) = WhiteFlag::get()
        .confirmation_sender
        .unbounded_send(LedgerWorkerEvent::Confirm(last_solid_milestone.0.clone()))
    {
        warn!(
            "Sending solid milestone {:?} to confirmation failed: {:?}.",
            last_solid_milestone.0.index(),
            e
        );
    }
}

pub fn init(
    index: u32,
    // TODO get concrete type
    state: HashMap<Address, u64>,
    bus: Arc<Bus<'static>>,
    shutdown: &mut Shutdown,
) -> mpsc::UnboundedSender<LedgerWorkerEvent> {
    // TODO
    // if unsafe { !WHITE_FLAG.is_null() } {
    //     warn!("Already initialized.");
    //     return;
    // }

    let (ledger_worker_tx, ledger_worker_rx) = mpsc::unbounded();
    let (ledger_worker_shutdown_tx, ledger_worker_shutdown_rx) = oneshot::channel();

    shutdown.add_worker_shutdown(
        ledger_worker_shutdown_tx,
        spawn(
            LedgerWorker::new(
                MilestoneIndex(index),
                state,
                ShutdownStream::new(ledger_worker_shutdown_rx, ledger_worker_rx),
            )
            .run(),
        ),
    );

    let white_flag = WhiteFlag {
        bus: bus.clone(),
        confirmation_sender: ledger_worker_tx.clone(),
    };

    unsafe {
        WHITE_FLAG = Box::leak(white_flag.into()) as *const _;
    }

    bus.add_listener(on_last_solid_milestone_changed);

    ledger_worker_tx
}

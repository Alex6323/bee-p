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

pub(crate) use confirmation::{LedgerConfirmationWorker, LedgerConfirmationWorkerEvent};

use bee_common::{shutdown::Shutdown, shutdown_stream::ShutdownStream};
use bee_common_ext::event::Bus;
use bee_protocol::{event::LastSolidMilestoneChanged, MilestoneIndex};

use async_std::task::spawn;
use futures::channel::{mpsc, oneshot};
use log::warn;

use std::{ptr, sync::Arc};

struct WhiteFlag {
    pub(crate) bus: Arc<Bus<'static>>,
    confirmation_sender: mpsc::UnboundedSender<LedgerConfirmationWorkerEvent>,
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
        .unbounded_send(LedgerConfirmationWorkerEvent(last_solid_milestone.0.clone()))
    {
        warn!(
            "Sending solid milestone {:?} to confirmation failed: {:?}.",
            last_solid_milestone.0.index(),
            e
        );
    }
}

pub(crate) fn init(snapshot_index: u32, bus: Arc<Bus<'static>>, shutdown: &mut Shutdown) {
    if unsafe { !WHITE_FLAG.is_null() } {
        warn!("Already initialized.");
        return;
    }

    let (ledger_confirmation_worker_tx, ledger_confirmation_worker_rx) = mpsc::unbounded();
    let (ledger_confirmation_worker_shutdown_tx, ledger_confirmation_worker_shutdown_rx) = oneshot::channel();

    shutdown.add_worker_shutdown(
        ledger_confirmation_worker_shutdown_tx,
        spawn(
            LedgerConfirmationWorker::new(
                MilestoneIndex(snapshot_index),
                ShutdownStream::new(ledger_confirmation_worker_shutdown_rx, ledger_confirmation_worker_rx),
            )
            .run(),
        ),
    );

    let white_flag = WhiteFlag {
        bus: bus.clone(),
        confirmation_sender: ledger_confirmation_worker_tx,
    };

    unsafe {
        WHITE_FLAG = Box::leak(white_flag.into()) as *const _;
    }

    bus.add_listener(on_last_solid_milestone_changed);
}

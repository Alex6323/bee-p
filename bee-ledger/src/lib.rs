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

mod merkle;
mod workers;

pub(crate) use merkle::Merkle;
pub use workers::{LedgerStateWorker, LedgerStateWorkerEvent};

use bee_common::shutdown::Shutdown;
use bee_transaction::bundled::Address;

use async_std::task::spawn;
use futures::channel::{mpsc, oneshot};

use std::collections::HashMap;

// TODO get concrete type
pub fn init(state: HashMap<Address, u64>, shutdown: &mut Shutdown) -> mpsc::Sender<LedgerStateWorkerEvent> {
    // TODO config
    let (ledger_worker_tx, ledger_worker_rx) = mpsc::channel(1000);
    let (ledger_worker_shutdown_tx, ledger_worker_shutdown_rx) = oneshot::channel();

    shutdown.add_worker_shutdown(
        ledger_worker_shutdown_tx,
        spawn(LedgerStateWorker::new(state).run(ledger_worker_rx, ledger_worker_shutdown_rx)),
    );

    ledger_worker_tx
}

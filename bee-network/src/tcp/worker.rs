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

use crate::{
    address::Address,
    endpoint::{allowlist, origin::Origin},
    events::EventPublisher as Notifier,
};

use super::{connection::TcpConnection, spawn_connection_workers};

use bee_common::{shutdown::ShutdownListener as Shutdown, worker::Error as WorkerError};

use async_std::net::TcpListener;
use futures::{prelude::*, select};
use log::*;

pub(crate) struct TcpWorker {
    binding_addr: Address,
    notifier: Notifier,
    shutdown: Shutdown,
}

impl TcpWorker {
    pub fn new(binding_addr: Address, notifier: Notifier, shutdown: Shutdown) -> Self {
        Self {
            binding_addr,
            notifier,
            shutdown,
        }
    }

    pub async fn run(mut self) -> Result<(), WorkerError> {
        debug!("Starting TCP worker...");

        let listener = TcpListener::bind(*self.binding_addr).await?;

        info!("Accepting connections on {}.", listener.local_addr()?);

        let mut incoming = listener.incoming().fuse();
        let shutdown = &mut self.shutdown;

        loop {
            select! {
                stream = incoming.next() => {
                    if let Some(stream) = stream {
                        match stream {
                            Ok(stream) => {

                                let conn = match TcpConnection::new(stream, Origin::Inbound) {
                                    Ok(conn) => conn,
                                    Err(e) => {
                                        error!["Creating TCP connection failed: {:?}.", e];
                                        continue;
                                    }
                                };

                                let allowlist = allowlist::get();

                                // Update IP addresses if necessary
                                // allowlist.refresh().await;

                                // Immediatedly drop stream, if it's associated IP address isn't on the allowlist
                                if !allowlist.contains_address(&conn.remote_addr.ip()) {
                                    warn!("Contacted by unknown IP address '{}'.", &conn.remote_addr.ip());
                                    warn!("Connection disallowed.");
                                    continue;
                                }

                                info!(
                                    "Sucessfully established connection to {} ({}).",
                                    conn.remote_addr,
                                    Origin::Inbound
                                );

                                match spawn_connection_workers(conn, self.notifier.clone()).await {
                                    Ok(_) => (),
                                    Err(_) => (),
                                }
                            }
                            Err(e) => {
                                error!("Accepting connection failed: {:?}.", e);
                            },
                        }
                    } else {
                        break;
                    }
                },
                shutdown = shutdown.fuse() => {
                    break;
                }
            }
        }

        debug!("Stopped TCP worker.");
        Ok(())
    }
}

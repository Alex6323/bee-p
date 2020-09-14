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

use crate::{events::EventSender, tcp::Origin};

use super::{connection::Connection, spawn_connection_workers};

use bee_common::{shutdown::ShutdownListener, worker::Error as WorkerError};

use futures::{prelude::*, select};
use log::*;
use tokio::net::{TcpListener, TcpStream};

use std::{io::Error, net::SocketAddr};

pub(crate) struct TcpServer {
    binding_address: SocketAddr,
    internal_event_sender: EventSender,
    allowlist: Arc<Allowlist>,
}

impl TcpServer {
    pub fn new(binding_address: SocketAddr, internal_event_sender: EventSender) -> Self {
        Self {
            binding_address,
            internal_event_sender,
        }
    }

    pub async fn run(mut self, shutdown_listener: ShutdownListener) -> Result<(), WorkerError> {
        debug!("Starting TCP server...");

        let mut listener = TcpListener::bind(self.binding_address).await?;

        debug!("Accepting connections on {}.", listener.local_addr()?);

        let mut fused_incoming_streams = listener.incoming().fuse();
        let mut fused_shutdown_listener = shutdown_listener.fuse();

        loop {
            select! {
                _ = fused_shutdown_listener => {
                    break;
                },
                stream = fused_incoming_streams.next() => {
                    if let Some(stream) = stream {
                        if !self.process_stream(stream).await? {
                            continue;
                        }
                    } else {
                        break;
                    }
                },
            }
        }

        debug!("Stopped TCP server.");
        Ok(())
    }

    #[inline]
    async fn process_stream(&mut self, stream: Result<TcpStream, Error>) -> Result<bool, WorkerError> {
        match stream {
            Ok(stream) => {
                let conn = match Connection::new(stream, Origin::Inbound) {
                    Ok(conn) => conn,
                    Err(e) => {
                        warn!("Creating TCP connection failed: {:?}.", e);

                        return Ok(false);
                    }
                };

                let allowlist = allowlist::get();

                // Immediatedly drop stream, if it's associated IP address isn't part of the allowlist
                if !allowlist.contains(&conn.peer_address.ip()) {
                    warn!("Contacted by unknown IP address '{}'.", &conn.peer_address.ip());
                    warn!("Connection disallowed.");

                    return Ok(false);
                }

                debug!(
                    "Sucessfully established connection to {} ({}).",
                    conn.peer_address,
                    Origin::Inbound
                );

                spawn_connection_workers(conn, self.internal_event_sender.clone())
                    .await
                    .unwrap_or_else(|_| ());
            }
            Err(e) => {
                warn!("Accepting connection failed: {:?}.", e);
            }
        }

        Ok(true)
    }
}

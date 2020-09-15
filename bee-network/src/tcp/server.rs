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
    endpoint::{Allowlist, EndpointId},
    events::EventSender,
    tcp::Origin,
};

use super::{connection::Connection, spawn_reader_writer};

use bee_common::{shutdown::ShutdownListener, worker::Error as WorkerError};

use futures::{prelude::*, select};
use log::*;
use tokio::net::{TcpListener, TcpStream};

use std::{io::Error, net::SocketAddr};

pub struct TcpServer {
    binding_address: SocketAddr,
    internal_event_sender: EventSender,
    allowlist: Allowlist,
    tcp_listener: TcpListener,
    shutdown_listener: ShutdownListener,
}

impl TcpServer {
    pub async fn new(
        binding_address: SocketAddr,
        internal_event_sender: EventSender,
        shutdown_listener: ShutdownListener,
        allowlist: Allowlist,
    ) -> Self {
        debug!("Starting TCP server...");

        let tcp_listener = TcpListener::bind(binding_address.clone())
            .await
            .expect("Error binding TCP server");

        debug!(
            "Accepting connections on {}.",
            tcp_listener.local_addr().expect("Error starting TCP server.")
        );

        Self {
            binding_address,
            internal_event_sender,
            allowlist,
            tcp_listener,
            shutdown_listener,
        }
    }

    pub async fn run(self) -> Result<(), WorkerError> {
        let TcpServer {
            mut tcp_listener,
            internal_event_sender,
            allowlist,
            shutdown_listener,
            ..
        } = self;

        let mut fused_incoming_streams = tcp_listener.incoming().fuse();
        let mut fused_shutdown_listener = shutdown_listener.fuse();

        loop {
            select! {
                _ = fused_shutdown_listener => {
                    break;
                },
                stream = fused_incoming_streams.next() => {
                    if let Some(stream) = stream {
                        if !process_stream(stream, &allowlist, &internal_event_sender).await? {
                            continue;
                        }
                    } else {
                        break;
                    }
                },
            }
        }

        debug!("TCP server stopped.");
        Ok(())
    }
}

#[inline]
async fn process_stream(
    stream: Result<TcpStream, Error>,
    allowlist: &Allowlist,
    internal_event_sender: &EventSender,
) -> Result<bool, WorkerError> {
    match stream {
        Ok(stream) => {
            let connection = match Connection::new(stream, Origin::Inbound) {
                Ok(conn) => conn,
                Err(e) => {
                    warn!("Creating TCP connection failed: {:?}.", e);

                    return Ok(false);
                }
            };

            let ip_address = connection.peer_address.ip();
            if !allowlist.allows(&ip_address) {
                warn!(
                    "Contacted by disallowed IP address '{}'.",
                    &connection.peer_address.ip()
                );
                warn!("Connection dropped.");

                return Ok(false);
            }

            debug!(
                "Sucessfully established connection to {} ({}).",
                connection.peer_address,
                Origin::Inbound
            );

            let internal_event_sender = internal_event_sender.clone();
            let epid = EndpointId::new();

            spawn_reader_writer(connection, epid, internal_event_sender)
                .await
                .map_err(|_| WorkerError::AsynchronousOperationFailed);
        }
        Err(e) => {
            warn!("Accepting connection failed: {:?}.", e);
        }
    }

    Ok(true)
}

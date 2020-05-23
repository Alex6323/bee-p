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

pub mod connection;
pub mod worker;

use connection::TcpConnection;

use crate::{
    address::{url::Protocol, Address},
    constants::MAX_BUFFER_SIZE,
    endpoint::{
        origin::Origin,
        outbox::{bytes_channel, BytesReceiver},
        Endpoint, EndpointId as EpId,
    },
    errors::{ConnectionError, ConnectionResult},
    events::{Event, EventPublisher as Notifier},
};

use async_std::{net::TcpStream, sync::Arc, task::spawn};
use futures::{channel::oneshot, prelude::*, select};
use log::*;

/// Tries to connect to an endpoint.
pub(crate) async fn try_connect(epid: &EpId, addr: &Address, notifier: Notifier) -> ConnectionResult<()> {
    info!("Trying to connect to {}...", epid);

    match TcpStream::connect(**addr).await {
        Ok(stream) => {
            let conn = match TcpConnection::new(stream, Origin::Outbound) {
                Ok(conn) => conn,
                Err(e) => {
                    error!["Error creating TCP connection: {:?}.", e];
                    return Err(ConnectionError::ConnectionAttemptFailed);
                }
            };

            info!(
                "Sucessfully established connection to {} ({}).",
                conn.remote_addr,
                Origin::Outbound
            );

            Ok(spawn_connection_workers(conn, notifier).await?)
        }
        Err(e) => {
            warn!("Connecting to {} failed: {:?}.", epid, e);
            Err(ConnectionError::ConnectionAttemptFailed)
        }
    }
}

pub(crate) async fn spawn_connection_workers(conn: TcpConnection, mut notifier: Notifier) -> ConnectionResult<()> {
    debug!("Spawning TCP connection workers...");

    let addr: Address = conn.remote_addr.into();
    let proto = Protocol::Tcp;
    let origin = conn.origin;

    let ep = Endpoint::new(addr, proto);

    let (sender, receiver) = bytes_channel();
    let (shutdown_sender, shutdown_receiver) = oneshot::channel::<()>();

    spawn(writer(ep.id, conn.stream.clone(), receiver, shutdown_sender));
    spawn(reader(ep.id, conn.stream.clone(), notifier.clone(), shutdown_receiver));

    Ok(notifier.send(Event::NewConnection { ep, origin, sender }).await?)
}

async fn writer(epid: EpId, stream: Arc<TcpStream>, bytes_rx: BytesReceiver, sd: oneshot::Sender<()>) {
    debug!("Starting connection writer task for {}...", epid);

    let mut stream = &*stream;
    let mut bytes_rx = bytes_rx.fuse();

    loop {
        select! {
            bytes_out = bytes_rx.next() => {
                if let Some(bytes_out) = bytes_out {

                    match stream.write_all(&*bytes_out).await {
                        Ok(_) => {
                            // NOTE: if we should need it, we can raise [`Event::BytesSent`] here.
                        },
                        Err(e) => {
                            error!("Sending bytes failed: {:?}.", e);
                        }
                    }
                } else {
                    // NOTE: If the bytes sender gets dropped (which happens when the connection pool
                    // is dropped, we break out of the loop)
                    break;
                }
            }
        }
    }

    if sd.send(()).is_err() {
        trace!("Reader task shut down before writer task.");
    }

    debug!("Connection writer event loop for {} stopped.", epid);
}

async fn reader(epid: EpId, stream: Arc<TcpStream>, mut notifier: Notifier, mut sd: oneshot::Receiver<()>) {
    debug!("Starting connection reader event loop for {}...", epid);

    let mut stream = &*stream;
    let mut buffer = vec![0; MAX_BUFFER_SIZE];
    let shutdown = &mut sd;

    loop {
        select! {
            num_read = stream.read(&mut buffer).fuse() => {
                match num_read {
                    Ok(num_read) => {
                        if num_read == 0 {
                            trace!("Received EOF (0 byte message).");

                            if notifier.send(Event::LostConnection { epid }).await.is_err() {
                                warn!("Failed to send 'LostConnection' notification.");
                            }

                            // NOTE: local reader shut down first (we were disconnected)
                            break;
                        } else {
                            let mut bytes = vec![0u8; num_read];
                            bytes.copy_from_slice(&buffer[0..num_read]);

                            if notifier.send(Event::MessageReceived { epid, bytes }).await.is_err() {
                                warn!("Failed to send 'MessageReceived' notification.");
                            }
                        }
                    },
                    Err(e) => {
                        error!("Receiveing bytes failed: {:?}.", e);
                    }
                }
            },
            shutdown = shutdown.fuse() => {
                // NOTE: local writer shut down first (we disconnected)
                break;
            }
        }
    }
    debug!("Connection reader event loop for {} stopped.", epid);
}

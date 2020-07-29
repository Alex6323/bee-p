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

use bee_common::shutdown::{ShutdownListener, ShutdownNotifier};

use crate::{
    address::{url::Protocol, Address},
    constants::MAX_BUFFER_SIZE,
    endpoint::{
        origin::Origin,
        outbox::{bytes_channel, BytesReceiver},
        Endpoint, EndpointId as EpId,
    },
    errors::{ConnectionError, ConnectionResult},
    events::{Event, EventPublisher},
};

use async_std::{net::TcpStream, sync::Arc, task::spawn};
use futures::{channel::oneshot, prelude::*, select, StreamExt};
use log::*;

// TODO: get rid of `ConnectionResult`

pub(crate) async fn try_connect(epid: &EpId, addr: &Address, event_pub_intern: EventPublisher) -> ConnectionResult<()> {
    debug!("Trying to connect to {}...", epid);

    match TcpStream::connect(**addr).await {
        Ok(stream) => {
            let conn = match TcpConnection::new(stream, Origin::Outbound) {
                Ok(conn) => conn,
                Err(e) => {
                    warn!["Error creating TCP connection: {:?}.", e];

                    return Err(ConnectionError::ConnectionAttemptFailed);
                }
            };

            debug!(
                "Sucessfully established connection to {} ({}).",
                conn.remote_addr,
                Origin::Outbound
            );

            Ok(spawn_connection_workers(conn, event_pub_intern).await?)
        }
        Err(e) => {
            warn!("Connecting to {} failed: {:?}.", epid, e);

            Err(ConnectionError::ConnectionAttemptFailed)
        }
    }
}

pub(crate) async fn spawn_connection_workers(
    conn: TcpConnection,
    mut event_pub_intern: EventPublisher,
) -> ConnectionResult<()> {
    debug!("Spawning TCP connection workers...");

    let addr: Address = conn.remote_addr.into();
    let proto = Protocol::Tcp;
    let origin = conn.origin;

    let ep = Endpoint::new(addr, proto);

    let (sender, receiver) = bytes_channel();
    let (shutdown_sender, shutdown_receiver) = oneshot::channel::<()>();

    spawn(writer(ep.id, conn.stream.clone(), receiver, shutdown_sender));
    spawn(reader(
        ep.id,
        conn.stream.clone(),
        event_pub_intern.clone(),
        shutdown_receiver,
    ));

    Ok(event_pub_intern
        .send(Event::NewConnection { ep, origin, sender })
        .await?)
}

// TODO: error handling

async fn writer(
    epid: EpId,
    stream: Arc<TcpStream>,
    bytes_receiver: BytesReceiver,
    shutdown_notifier: ShutdownNotifier,
) {
    debug!("Starting connection writer task for {}...", epid);

    let mut stream = &*stream;
    let mut fused_bytes_receiver = bytes_receiver.fuse();

    loop {
        let bytes = fused_bytes_receiver.next().await;
        if let Some(bytes) = bytes {
            stream
                .write_all(&*bytes)
                .await
                .unwrap_or_else(|e| error!("Sending bytes failed: {:?}", e));
        } else {
            // NOTE: If the bytes sender gets dropped (which happens when the connection pool
            // is dropped, we break out of the loop)
            break;
        }
    }

    // Try to send the shutdown signal, but don't care about whether it succeeds, since if it fails,  the receiver was
    // shut down first.
    shutdown_notifier.send(()).map_err(|_| ()).unwrap();

    debug!("Connection writer event loop for {} stopped.", epid);
}

async fn reader(
    epid: EpId,
    stream: Arc<TcpStream>,
    mut event_pub_intern: EventPublisher,
    shutdown_listener: ShutdownListener,
) {
    debug!("Starting connection reader event loop for {}...", epid);

    let mut buffer = vec![0u8; MAX_BUFFER_SIZE];

    let mut stream = &*stream;
    // let mut fused_stream_read = stream.read(&mut buffer).fuse();
    let mut fused_shutdown = &mut shutdown_listener.fuse();

    loop {
        select! {
            num_read = stream.read(&mut buffer).fuse() => {
            //num_read = fused_stream_read => {
                match num_read {
                    Ok(num_read) => {
                        if !handle_read(epid, num_read, &mut event_pub_intern, &buffer).await {
                            break;
                        }
                    },
                    Err(e) => {
                        warn!("Receiving bytes failed: {:?}.", e);
                    }
                }
            },
            _ = fused_shutdown => {
                // NOTE: local writer shut down first (we disconnected)
                break;
            }
        }
    }

    debug!("Connection reader event loop for {} stopped.", epid);
}

#[inline]
async fn handle_read(epid: EpId, num_read: usize, event_pub_intern: &mut EventPublisher, buffer: &Vec<u8>) -> bool {
    if num_read == 0 {
        debug!("Received EOF (0 byte message).");

        if event_pub_intern.send(Event::LostConnection { epid }).await.is_err() {
            warn!("Failed to inform about lost connection.");
        }

        // NOTE: local reader shut down first (we were disconnected)
        // break;
        false
    } else {
        let mut bytes = vec![0u8; num_read];
        bytes.copy_from_slice(&buffer[0..num_read]);

        if event_pub_intern
            .send(Event::MessageReceived { epid, bytes })
            .await
            .is_err()
        {
            warn!("Failed to notify about received message.");
        }
        true
    }
}

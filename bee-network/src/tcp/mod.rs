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
    endpoint::{
        origin::Origin,
        outbox::{bytes_channel, BytesReceiver},
        Endpoint, EndpointId as EpId,
    },
    events::{Event, EventSender},
};

use async_std::{
    io::prelude::{ReadExt, WriteExt},
    net::TcpStream,
    sync::Arc,
    task::spawn,
};
use futures::{channel::oneshot, future::FutureExt, select, sink::SinkExt, StreamExt};
use log::*;
use thiserror::Error;

const MAX_BUFFER_SIZE: usize = 1654;

#[derive(Debug, Error)]
pub enum Error {
    #[error("An async I/O error occured.")]
    AsyncIoErrorOccurred(#[from] std::io::Error),

    #[error("Connection attempt failed.")]
    ConnectionAttemptFailed,

    #[error("Sending an event failed.")]
    SendingEventFailed(#[from] futures::channel::mpsc::SendError),
}

pub(crate) async fn try_connect(epid: &EpId, addr: &Address, internal_event_sender: EventSender) -> Result<(), Error> {
    debug!("Trying to connect to {}...", epid);

    match TcpStream::connect(**addr).await {
        Ok(stream) => {
            let conn = match TcpConnection::new(stream, Origin::Outbound) {
                Ok(conn) => conn,
                Err(e) => {
                    warn!["Error creating TCP connection: {:?}.", e];

                    return Err(Error::ConnectionAttemptFailed);
                }
            };

            debug!(
                "Sucessfully established connection to {} ({}).",
                conn.remote_addr,
                Origin::Outbound
            );

            Ok(spawn_connection_workers(conn, internal_event_sender).await?)
        }
        Err(e) => {
            warn!("Connecting to {} failed: {:?}.", epid, e);

            Err(Error::ConnectionAttemptFailed)
        }
    }
}

pub(crate) async fn spawn_connection_workers(
    conn: TcpConnection,
    mut internal_event_sender: EventSender,
) -> Result<(), Error> {
    debug!("Spawning TCP connection workers...");

    let addr: Address = conn.remote_addr.into();
    let proto = Protocol::Tcp;
    let origin = conn.origin;

    let endpoint = Endpoint::new(addr, proto);

    let (bytes_sender, bytes_receiver) = bytes_channel();
    let (shutdown_sender, shutdown_receiver) = oneshot::channel::<()>();

    spawn(writer(
        endpoint.id,
        conn.stream.clone(),
        bytes_receiver,
        shutdown_sender,
    ));

    spawn(reader(
        endpoint.id,
        conn.stream.clone(),
        internal_event_sender.clone(),
        shutdown_receiver,
    ));

    Ok(internal_event_sender
        .send(Event::NewConnection {
            endpoint,
            origin,
            sender: bytes_sender,
        })
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
            break;
        }
    }

    // Try to send the shutdown signal, but don't care about whether it succeeds, since if it fails,  the receiver was
    // shut down first.
    shutdown_notifier.send(()).unwrap_or_else(|_| ());

    debug!("Connection writer event loop for {} stopped.", epid);
}

async fn reader(
    epid: EpId,
    stream: Arc<TcpStream>,
    mut internal_event_sender: EventSender,
    shutdown_listener: ShutdownListener,
) {
    debug!("Starting connection reader event loop for {}...", epid);

    let mut buffer = vec![0u8; MAX_BUFFER_SIZE];

    let mut stream = &*stream;
    // let mut fused_stream_read = stream.read(&mut buffer).fuse();
    let mut fused_shutdown = &mut shutdown_listener.fuse();

    loop {
        select! {
            _ = fused_shutdown => {
                break;
            }
            num_read = stream.read(&mut buffer).fuse() => {
                match num_read {
                    Ok(num_read) => {
                        if !handle_read(epid, num_read, &mut internal_event_sender, &buffer).await {
                            break;
                        }
                    },
                    Err(e) => {
                        warn!("Receiving bytes failed: {:?}.", e);
                    }
                }
            },
        }
    }

    debug!("Connection reader event loop for {} stopped.", epid);
}

#[inline]
async fn handle_read(epid: EpId, num_read: usize, internal_event_sender: &mut EventSender, buffer: &Vec<u8>) -> bool {
    if num_read == 0 {
        debug!("Received EOF (0 byte message).");

        if internal_event_sender
            .send(Event::LostConnection { epid })
            .await
            .is_err()
        {
            warn!("Failed to inform about lost connection.");
        }

        // local reader shut down first (we were disconnected)
        false
    } else {
        let mut bytes = vec![0u8; num_read];
        bytes.copy_from_slice(&buffer[0..num_read]);

        if internal_event_sender
            .send(Event::MessageReceived { epid, bytes })
            .await
            .is_err()
        {
            warn!("Failed to notify about received message.");
        }
        true
    }
}

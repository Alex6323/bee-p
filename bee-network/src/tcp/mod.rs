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

pub mod client;
pub mod connection;
pub mod server;

use connection::{Connection, Origin};

use bee_common::shutdown::{ShutdownListener, ShutdownNotifier};

use crate::{
    endpoint::{channel, contact::EndpointContactParams, DataReceiver, EndpointId, TransportProtocol},
    events::{Event, EventSender},
    MAX_TCP_BUFFER_SIZE,
};

use futures::{
    channel::oneshot,
    future::{join_all, FutureExt},
    select,
    sink::SinkExt,
    StreamExt,
};
use log::*;
use thiserror::Error;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::tcp::{OwnedReadHalf, OwnedWriteHalf},
    task::JoinHandle,
};

use std::{net::SocketAddr, sync::atomic::Ordering};

#[derive(Debug, Error)]
pub enum Error {
    #[error("An async I/O error occured.")]
    AsyncIoErrorOccurred(#[from] std::io::Error),

    #[error("Connection attempt failed.")]
    ConnectionAttemptFailed,

    #[error("Sending an event failed.")]
    SendingEventFailed(#[from] futures::channel::mpsc::SendError),
}

pub(crate) async fn spawn_connection_workers(
    connection: Connection,
    mut internal_event_sender: EventSender,
) -> Result<(), Error> {
    debug!("Spawning TCP connection workers...");

    let Connection {
        origin,
        own_address,
        peer_address,
        reader,
        writer,
    } = connection;

    let transport_protocol = TransportProtocol::Tcp;

    let endpoint = EndpointContactParams::from_socket_address(socket_address, transport_protocol);

    let (data_sender, data_receiver) = channel();
    let (shutdown_sender, shutdown_receiver) = oneshot::channel::<()>();

    // NOTE: block until reader and writer task are spawned
    let mut handles = Vec::with_capacity(2);

    handles.push(spawn_writer(endpoint.epid, writer, data_receiver, shutdown_sender));
    handles.push(spawn_reader(
        endpoint.epid,
        reader,
        internal_event_sender.clone(),
        shutdown_receiver,
    ));

    join_all(handles).await;

    internal_event_sender
        .send(Event::ConnectionCreated {
            endpoint,
            origin,
            data_sender,
            timestamp: crate::utils::time::timestamp_millis(),
        })
        .await?;

    Ok(())
}

fn spawn_writer(
    epid: EndpointId,
    mut writer: OwnedWriteHalf,
    data_receiver: DataReceiver,
    shutdown_notifier: ShutdownNotifier,
) -> JoinHandle<()> {
    debug!("Starting connection writer task for {}...", epid);

    let mut fused_data_receiver = data_receiver.fuse();

    tokio::spawn(async move {
        // let mut stream = &*stream;

        loop {
            let data = fused_data_receiver.next().await;

            if let Some(bytes) = data {
                writer
                    .write_all(&*bytes)
                    // .fuse() // necessary?
                    .await
                    .unwrap_or_else(|e| error!("Sending bytes failed: {:?}", e));
            } else {
                break;
            }
        }

        shutdown_notifier.send(()).unwrap_or_else(|_| ());

        debug!("Connection writer loop for {} stopped.", epid);
    })
}

fn spawn_reader(
    epid: EndpointId,
    mut reader: OwnedReadHalf,
    mut internal_event_sender: EventSender,
    shutdown_listener: ShutdownListener,
) -> JoinHandle<()> {
    debug!("Starting connection reader task for {}...", epid);

    let mut buffer = vec![0u8; MAX_TCP_BUFFER_SIZE.load(Ordering::Relaxed)];

    tokio::spawn(async move {
        // let mut stream = &mut *stream;
        let mut fused_shutdown = &mut shutdown_listener.fuse();

        loop {
            select! {
                _ = fused_shutdown => {
                    break;
                }
                num_read = reader.read(&mut buffer).fuse() => {
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

        debug!("Connection reader loop for {} stopped.", epid);
    })
}

#[inline]
async fn handle_read(
    epid: EndpointId,
    num_read: usize,
    internal_event_sender: &mut EventSender,
    buffer: &Vec<u8>,
) -> bool {
    if num_read == 0 {
        debug!("Received EOF (0 byte message).");

        if internal_event_sender
            .send(Event::ConnectionDropped { epid })
            .await
            .is_err()
        {
            warn!("Failed to inform about lost connection.");
        }

        false
    } else {
        let mut message = vec![0u8; num_read];
        message.copy_from_slice(&buffer[0..num_read]);

        if internal_event_sender
            .send(Event::MessageReceived { epid, message })
            .await
            .is_err()
        {
            warn!("Failed to notify about received message.");
        }

        true
    }
}

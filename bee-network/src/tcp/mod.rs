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

use connection::{Connection, Origin};

use bee_common::shutdown::{ShutdownListener, ShutdownNotifier};

use crate::{
    address::{url::Protocol, Address},
    endpoint::{
        connected::{channel, DataReceiver},
        Endpoint, EndpointId,
    },
    events::{Event, EventSender},
    MAX_TCP_BUFFER_SIZE,
};

use async_std::{
    io::prelude::{ReadExt, WriteExt},
    net::TcpStream,
    sync::Arc,
    task::{block_on, spawn},
};
use futures::{channel::oneshot, future::FutureExt, select, sink::SinkExt, StreamExt};
use log::*;
use thiserror::Error;

use std::sync::atomic::Ordering;

#[derive(Debug, Error)]
pub enum Error {
    #[error("An async I/O error occured.")]
    AsyncIoErrorOccurred(#[from] std::io::Error),

    #[error("Connection attempt failed.")]
    ConnectionAttemptFailed,

    #[error("Sending an event failed.")]
    SendingEventFailed(#[from] futures::channel::mpsc::SendError),
}

pub(crate) async fn try_connect_to(
    epid: &EndpointId,
    address: &Address,
    internal_event_sender: EventSender,
) -> Result<(), Error> {
    debug!("Trying to connect to {}...", epid);

    match TcpStream::connect(**address).await {
        Ok(stream) => {
            let connection = match Connection::new(stream, Origin::Outbound) {
                Ok(conn) => conn,
                Err(e) => {
                    warn!["Error creating TCP connection: {:?}.", e];

                    return Err(Error::ConnectionAttemptFailed);
                }
            };

            debug!(
                "Sucessfully established connection to {} ({}).",
                connection.remote_addr, connection.origin,
            );

            spawn_connection_workers(connection, internal_event_sender).await?;

            Ok(())
        }
        Err(e) => {
            warn!("Connecting to {} failed: {:?}.", epid, e);

            Err(Error::ConnectionAttemptFailed)
        }
    }
}

pub(crate) async fn spawn_connection_workers(
    connection: Connection,
    mut internal_event_sender: EventSender,
) -> Result<(), Error> {
    debug!("Spawning TCP connection workers...");

    let address: Address = connection.remote_addr.into();
    let protocol = Protocol::Tcp;
    let origin = connection.origin;
    let timestamp = connection.timestamp;

    let endpoint = Endpoint::new(address, protocol);

    let (data_sender, data_receiver) = channel();
    let (shutdown_sender, shutdown_receiver) = oneshot::channel::<()>();

    spawn_writer(endpoint.epid, connection.stream.clone(), data_receiver, shutdown_sender);

    spawn_reader(
        endpoint.epid,
        connection.stream.clone(),
        internal_event_sender.clone(),
        shutdown_receiver,
    );

    // NOTE: wait a little to make sure the reader/writer workers are waiting for events
    // async_std::task::sleep(std::time::Duration::from_millis(1)).await;

    block_on(internal_event_sender.send(Event::ConnectionCreated {
        endpoint,
        origin,
        data_sender,
        timestamp,
    }))?;

    Ok(())
}

fn spawn_writer(
    epid: EndpointId,
    stream: Arc<TcpStream>,
    data_receiver: DataReceiver,
    shutdown_notifier: ShutdownNotifier,
) {
    debug!("Starting connection writer task for {}...", epid);

    let mut fused_data_receiver = data_receiver.fuse();

    spawn(async move {
        let mut stream = &*stream;
        loop {
            let data = fused_data_receiver.next().await;

            if let Some(bytes) = data {
                stream
                    .write_all(&*bytes)
                    .await
                    .unwrap_or_else(|e| error!("Sending bytes failed: {:?}", e));
            } else {
                break;
            }
        }
        shutdown_notifier.send(()).unwrap_or_else(|_| ());

        debug!("Connection writer loop for {} stopped.", epid);
    });
}

fn spawn_reader(
    epid: EndpointId,
    stream: Arc<TcpStream>,
    mut internal_event_sender: EventSender,
    shutdown_listener: ShutdownListener,
) {
    debug!("Starting connection reader task for {}...", epid);

    let mut buffer = vec![0u8; MAX_TCP_BUFFER_SIZE.load(Ordering::Relaxed)];

    spawn(async move {
        let mut stream = &*stream;
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
        debug!("Connection reader loop for {} stopped.", epid);
    });
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

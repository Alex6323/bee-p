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

mod client;
mod connection;
mod server;

pub use client::*;
pub use connection::*;
pub use server::*;

use crate::{
    endpoint::{self, DataReceiver, EndpointId},
    event::{Event, EventSender},
    MAX_TCP_BUFFER_SIZE,
};

use bee_common::shutdown::{ShutdownListener, ShutdownNotifier};

use futures::{channel::oneshot, future::FutureExt, select, StreamExt};
use log::*;
use thiserror::Error as ErrorAttr;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::tcp::{OwnedReadHalf, OwnedWriteHalf},
    task::JoinHandle,
};

use std::sync::atomic::Ordering;

#[derive(Debug, ErrorAttr)]
pub enum Error {
    #[error("An async I/O error occured.")]
    IoError(#[from] std::io::Error),

    #[error("Connection attempt failed.")]
    ConnectionAttemptFailed,

    #[error("Sending an event failed.")]
    SendingEventFailed,
}

pub(crate) async fn spawn_reader_writer(
    connection: Connection,
    epid: EndpointId,
    internal_event_sender: EventSender,
) -> Result<(), Error> {
    let Connection {
        origin,
        own_address,
        peer_address,
        reader,
        writer,
    } = connection;

    let (data_sender, data_receiver) = endpoint::channel();
    let (shutdown_notifier, shutdown_listener) = oneshot::channel::<()>();

    spawn_writer(epid, writer, data_receiver, shutdown_notifier);
    spawn_reader(epid, reader, internal_event_sender.clone(), shutdown_listener);

    internal_event_sender
        .send_async(Event::ConnectionEstablished {
            epid,
            peer_address,
            origin,
            data_sender,
        })
        .await
        .map_err(|_| Error::SendingEventFailed)?;

    Ok(())
}

fn spawn_writer(
    epid: EndpointId,
    mut writer: OwnedWriteHalf,
    data_receiver: DataReceiver,
    shutdown_notifier: ShutdownNotifier,
) -> JoinHandle<()> {
    trace!("Starting TCP stream writer for {}...", epid);

    let mut fused_data_receiver = data_receiver.into_stream();

    tokio::spawn(async move {
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

        trace!("TCP stream writer for {} stopped.", epid);
    })
}

fn spawn_reader(
    epid: EndpointId,
    mut reader: OwnedReadHalf,
    mut internal_event_sender: EventSender,
    shutdown_listener: ShutdownListener,
) -> JoinHandle<()> {
    trace!("Starting TCP stream reader for {}...", epid);

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
                            if !process_stream_read(epid, num_read, &mut internal_event_sender, &buffer).await {
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

        trace!("TCP stream reader for {} stopped.", epid);
    })
}

#[inline]
async fn process_stream_read(
    epid: EndpointId,
    num_read: usize,
    internal_event_sender: &mut EventSender,
    buffer: &Vec<u8>,
) -> bool {
    if num_read == 0 {
        trace!("Stream dropped by peer (EOF).");

        if internal_event_sender
            .send_async(Event::ConnectionDropped { epid })
            .await
            .is_err()
        {
            warn!("Dropped internal event (OOM?)");
        }

        false
    } else {
        let mut message = vec![0u8; num_read];
        message.copy_from_slice(&buffer[0..num_read]);

        if internal_event_sender
            .send_async(Event::MessageReceived { epid, message })
            .await
            .is_err()
        {
            warn!("Dropped internal event (OOM?)");
        }

        true
    }
}

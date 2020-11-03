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

mod connection;
mod dial;
mod manager;

use crate::{
    interaction::events::{Event, EventSender},
    peers::{self, DataReceiver},
    MAX_BUFFER_SIZE,
};

pub use connection::Origin;
pub use dial::dial_peer;
pub use manager::ConnectionManager;

use connection::Connection;

use bee_common::shutdown::{ShutdownListener, ShutdownNotifier};

use futures::{channel::oneshot, prelude::*, select};
use libp2p::{
    core::muxing::{substream_from_ref, StreamMuxerBox},
    Multiaddr, PeerId,
};
use log::*;
use thiserror::Error as ErrorAttr;
use tokio::task::JoinHandle;

use std::{
    io,
    sync::{atomic::Ordering, Arc},
};

const SUBSTREAM_INDEX: usize = 0;

pub(crate) async fn spawn_reader_writer(
    connection: Connection,
    internal_event_sender: EventSender,
) -> Result<(), Error> {
    let Connection {
        peer_id,
        peer_address,
        stream,
        origin,
        ..
    } = connection;

    let stream = Arc::new(stream);

    let (data_sender, data_receiver) = peers::channel();
    let (shutdown_notifier, shutdown_listener) = oneshot::channel::<()>();

    spawn_writer(
        peer_id.clone(),
        peer_address.clone(),
        &stream,
        data_receiver,
        shutdown_notifier,
    );
    spawn_reader(
        peer_id.clone(),
        peer_address.clone(),
        &stream,
        internal_event_sender.clone(),
        shutdown_listener,
    );

    internal_event_sender
        .send_async(Event::ConnectionEstablished {
            peer_id,
            peer_address,
            origin,
            data_sender,
        })
        .await
        .map_err(|_| Error::SendingEventFailed)?;

    Ok(())
}

fn spawn_writer(
    peer_id: PeerId,
    peer_address: Multiaddr,
    mut stream: &Arc<StreamMuxerBox>,
    data_receiver: DataReceiver,
    shutdown_notifier: ShutdownNotifier,
) -> JoinHandle<()> {
    let mut stream = substream_from_ref(stream.clone(), SUBSTREAM_INDEX);
    let mut fused_data_receiver = data_receiver.into_stream();

    tokio::spawn(async move {
        loop {
            let message = fused_data_receiver.next().await;

            if let Some(message) = message {
                stream = match send_message(stream, &message).await {
                    Ok(stream) => stream,
                    Err(e) => {
                        error!("Sending message failed: {:?}", e);
                        break;
                    }
                }
            } else {
                // Data receiver closed => shutdown this task
                break;
            }
        }

        let _ = shutdown_notifier.send(());
    })
}

fn spawn_reader(
    peer_id: PeerId,
    peer_address: Multiaddr,
    mut stream: &Arc<StreamMuxerBox>,
    mut internal_event_sender: EventSender,
    shutdown_listener: ShutdownListener,
) -> JoinHandle<()> {
    let mut stream = substream_from_ref(stream.clone(), SUBSTREAM_INDEX);
    let mut buffer = vec![0u8; MAX_BUFFER_SIZE.load(Ordering::Relaxed)];

    tokio::spawn(async move {
        let mut fused_shutdown = &mut shutdown_listener.fuse();

        loop {
            select! {
                _ = fused_shutdown => {
                    break;
                }
                result = recv_message(stream, &mut buffer).fuse() => {
                    match result {
                        Ok((num_read, s)) => {
                            stream = s;

                            if !process_stream_read(
                                peer_id.clone(),
                                peer_address.clone(),
                                num_read,
                                &mut internal_event_sender,
                                &buffer)
                                .await
                            {
                                break;
                            }

                        }
                        Err(e) => {
                            error!("Receiving message failed: {:?}", e);
                            break;
                        }
                    }
                }
            }
        }
    })
}

#[inline]
async fn process_stream_read(
    peer_id: PeerId,
    peer_address: Multiaddr,
    num_read: usize,
    internal_event_sender: &mut EventSender,
    buffer: &[u8],
) -> bool {
    if num_read == 0 {
        trace!("Stream dropped by peer (EOF).");

        if internal_event_sender
            .send_async(Event::ConnectionDropped {
                peer_id: peer_id.clone(),
                peer_address: peer_address.clone(),
            })
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
            .send_async(Event::MessageReceived {
                peer_id: peer_id.clone(),
                message,
            })
            .await
            .is_err()
        {
            warn!("Dropped internal event (OOM?)");
        }

        true
    }
}

pub async fn send_message<S>(mut stream: S, message: &[u8]) -> io::Result<S>
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    stream.write_all(message).await?;
    stream.flush().await?;
    Ok(stream)
}

pub async fn recv_message<S>(mut stream: S, message: &mut [u8]) -> io::Result<(usize, S)>
where
    S: AsyncRead + Unpin,
{
    let num_read = stream.read(message).await?;
    Ok((num_read, stream))
}

#[derive(Debug, ErrorAttr)]
pub enum Error {
    #[error("An async I/O error occured.")]
    IoError(#[from] std::io::Error),

    #[error("Connection attempt failed.")]
    ConnectionAttemptFailed,

    #[error("Sending an event failed.")]
    SendingEventFailed,
}

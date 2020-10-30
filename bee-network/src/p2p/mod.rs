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
mod handler;
mod protocol;
mod server;

use crate::{
    endpoint::{self, DataReceiver},
    event::{Event, EventSender},
};

use connection::Connection;

use futures::{channel::oneshot, prelude::*};
use libp2p::core::muxing::StreamMuxerBox;
use thiserror::Error as ErrorAttr;
use tokio::task::JoinHandle;

use std::io;

pub(crate) async fn spawn_reader_writer(
    connection: Connection,
    internal_event_sender: EventSender,
) -> Result<(), Error> {
    let Connection {
        remote_id,
        remote_addr,
        stream,
        origin,
        ..
    } = connection;

    let (data_sender, data_receiver) = endpoint::channel();
    let (shutdown_notifier, shutdown_listener) = oneshot::channel::<()>();

    let mut fused_data_receiver = data_receiver.into_stream();

    tokio::spawn(async move {
        loop {
            let data = fused_data_receiver.next().await;

            if let Some(message) = data {
                //
                stream = match send_message(stream, message).await {
                    Ok(stream) => stream,
                    Err(e) => {
                        warn!("{:?}", e);
                        break;
                    }
                }
            } else {
                // Data receiver closed
                break;
            }
        }
    });
    // TODO: re-include

    // internal_event_sender
    //     .send_async(Event::ConnectionEstablished {
    //         remote_id,
    //         remote_addr,
    //         origin,
    //         data_sender,
    //     })
    //     .await
    //     .map_err(|_| Error::SendingEventFailed)?;

    Ok(())
}

// async_std::task::block_on(async move {
//             let c = MemoryTransport.dial(listener_addr).unwrap().await.unwrap();
//             let (_, rtt) = send_ping(c).await.unwrap();
//             assert!(rtt > Duration::from_secs(0));
//         });

fn spawn_writer(
    mut writer: StreamMuxerBox,
    data_receiver: DataReceiver,
    shutdown_notifier: ShutdownNotifier,
) -> JoinHandle<()> {
    let mut fused_data_receiver = data_receiver.into_stream();

    tokio::spawn(async move {
        loop {
            let data = fused_data_receiver.next().await;

            if let Some(bytes) = data {
                writer
                    .write_all(&*bytes)
                    .await
                    .unwrap_or_else(|e| error!("Sending bytes failed: {:?}", e));
            } else {
                break;
            }
        }

        let _ = shutdown_notifier.send(());
    })
}

fn spawn_reader(
    mut reader: StreamMuxerBox,
    mut internal_event_sender: EventSender,
    shutdown_listener: ShutdownListener,
) -> JoinHandle<()> {
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
    })
}

#[inline]
async fn process_stream_read(
    epid: EndpointId,
    num_read: usize,
    internal_event_sender: &mut EventSender,
    buffer: &[u8],
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

pub async fn send_message<S>(mut stream: S, message: &[u8]) -> io::Result<S>
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    stream.write_all(message).await?;
    stream.flush().await?;
    Ok(stream)
}

pub async fn recv_message<S>(mut stream: S, message: &mut [u8]) -> io::Result<S>
where
    S: AsyncRead + Unpin,
{
    stream.read_exact(message).await?;
    Ok(stream)
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

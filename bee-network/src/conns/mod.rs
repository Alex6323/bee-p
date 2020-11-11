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
    gossip::{GossipProtocol, NegotiatedSubstream},
    interaction::events::{EventSender, InternalEvent, InternalEventSender},
    peers::{self, DataReceiver},
    MSG_BUFFER_SIZE,
};

pub use connection::Origin;
pub use dial::dial;
pub use manager::ConnectionManager;

use connection::MuxedConnection;

use futures::{prelude::*, select, AsyncRead, AsyncWrite};
use libp2p::{
    core::{
        muxing::{event_from_ref_and_wrap, outbound_from_ref_and_wrap, StreamMuxerBox, SubstreamRef},
        upgrade,
    },
    Multiaddr, PeerId,
};
use log::*;
use thiserror::Error as ErrorAttr;
use tokio::task::JoinHandle;

use std::sync::{atomic::Ordering, Arc};

pub(crate) async fn spawn_connection_handler(
    connection: MuxedConnection,
    internal_event_sender: InternalEventSender,
) -> Result<(), Error> {
    let MuxedConnection {
        peer_id,
        peer_address,
        muxer,
        origin,
        ..
    } = connection;

    let muxer = Arc::new(muxer);
    let (message_sender, message_receiver) = peers::channel();

    let internal_event_sender_clone = internal_event_sender.clone();

    let substream = match origin {
        Origin::Outbound => {
            // FIXME: unwrap
            let outbound = outbound_from_ref_and_wrap(muxer).fuse().await.unwrap();
            // FIXME
            upgrade::apply_outbound(outbound, GossipProtocol, upgrade::Version::V1)
                .await
                .unwrap()
            // outbound
        }
        Origin::Inbound => {
            let inbound = loop {
                if let Some(substream) = event_from_ref_and_wrap(muxer.clone())
                    .await
                    .expect("error awaiting inbound substream")
                    .into_inbound_substream()
                {
                    break substream;
                }
            };

            upgrade::apply_inbound(inbound, GossipProtocol).await.unwrap()
            // inbound
        }
    };

    spawn_substream_task(
        peer_id.clone(),
        peer_address.clone(),
        substream,
        message_receiver,
        internal_event_sender_clone,
    );

    internal_event_sender
        .send_async(InternalEvent::ConnectionEstablished {
            peer_id,
            peer_address,
            origin,
            message_sender,
        })
        .await
        .map_err(|_| Error::SendingEventFailed)?;

    Ok(())
}

fn spawn_substream_task(
    peer_id: PeerId,
    peer_address: Multiaddr,
    mut substream: NegotiatedSubstream,
    message_receiver: DataReceiver,
    mut internal_event_sender: InternalEventSender,
) -> JoinHandle<()> {
    tokio::spawn(async move {
        let mut fused_message_receiver = message_receiver.into_stream();
        let mut buffer = vec![0u8; MSG_BUFFER_SIZE.load(Ordering::Relaxed)];

        loop {
            select! {
                num_read = recv_message(&mut substream, &mut buffer).fuse() => {
                    if !process_read(
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
                message = fused_message_receiver.next() => {
                    if let Some(message) = message {
                        send_message(&mut substream, &message).await;
                    } else {
                        // Data receiver closed => shutdown this task
                        break;
                    }

                }
            }
        }
    })
}

#[inline]
async fn send_message<S>(stream: &mut S, message: &[u8])
where
    S: AsyncWrite + Unpin,
{
    if let Err(e) = stream.write_all(message).await {
        warn!("Writing to stream failed due to {:?}", e);
        return;
    }
    if let Err(e) = stream.flush().await {
        warn!("Flushing a stream failed due to {:?}", e);
        return;
    }
    trace!("Wrote {} bytes to stream.", message.len());
}

#[inline]
async fn recv_message<S>(stream: &mut S, message: &mut [u8]) -> usize
where
    S: AsyncRead + Unpin,
{
    match stream.read(message).await {
        Ok(num_read) => {
            trace!("Read {} bytes from stream.", num_read);
            return num_read;
        }
        Err(e) => {
            warn!("Reading from a stream failed due to {:?}", e);
            return 0;
        }
    }
}

#[inline]
async fn process_read(
    peer_id: PeerId,
    peer_address: Multiaddr,
    num_read: usize,
    internal_event_sender: &mut InternalEventSender,
    buffer: &[u8],
) -> bool {
    if num_read == 0 {
        trace!("Stream dropped by peer (EOF).");

        if internal_event_sender
            .send_async(InternalEvent::ConnectionDropped {
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
            .send_async(InternalEvent::MessageReceived {
                message,
                from: peer_id.clone(),
            })
            .await
            .is_err()
        {
            warn!("Dropped internal event (OOM?)");
        }

        true
    }
}

#[derive(Debug, ErrorAttr)]
pub enum Error {
    #[error("An async I/O error occured.")]
    IoError(#[from] std::io::Error),

    #[error("Connection attempt failed.")]
    ConnectionAttemptFailed,

    #[error("Sending an event failed.")]
    SendingEventFailed,

    #[error("Upgrading the connection failed.")]
    ConnectionUpgradeError,
}

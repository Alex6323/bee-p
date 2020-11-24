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
mod errors;
mod manager;

use crate::{
    interaction::events::{InternalEvent, InternalEventSender},
    peers::{self, DataReceiver},
    protocols::gossip::{GossipProtocol, GossipSubstream},
    ReadableId, MSG_BUFFER_SIZE,
};

pub use connection::Origin;
pub use dial::dial;
pub use errors::Error;
pub use manager::ConnectionManager;

use connection::MuxedConnection;

use futures::{prelude::*, select, AsyncRead, AsyncWrite};
use libp2p::{
    core::{
        muxing::{event_from_ref_and_wrap, outbound_from_ref_and_wrap},
        upgrade,
    },
    Multiaddr, PeerId,
};
use log::*;
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
            let outbound = outbound_from_ref_and_wrap(muxer)
                .fuse()
                .await
                .map_err(|_| Error::CreatingOutboundSubstreamFailed(peer_id.readable()))?;

            upgrade::apply_outbound(outbound, GossipProtocol, upgrade::Version::V1)
                .await
                .map_err(|_| Error::SubstreamProtocolUpgradeFailed(peer_id.readable()))?
        }
        Origin::Inbound => {
            let inbound = loop {
                if let Some(inbound) = event_from_ref_and_wrap(muxer.clone())
                    .await
                    .map_err(|_| Error::CreatingInboundSubstreamFailed(peer_id.readable()))?
                    .into_inbound_substream()
                {
                    break inbound;
                }
            };

            upgrade::apply_inbound(inbound, GossipProtocol)
                .await
                .map_err(|_| Error::SubstreamProtocolUpgradeFailed(peer_id.readable()))?
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
        .map_err(|_| Error::InternalEventSendFailure("ConnectionEstablished"))?;

    Ok(())
}

fn spawn_substream_task(
    peer_id: PeerId,
    peer_address: Multiaddr,
    mut substream: GossipSubstream,
    message_receiver: DataReceiver,
    mut internal_event_sender: InternalEventSender,
) -> JoinHandle<()> {
    tokio::spawn(async move {
        let mut fused_message_receiver = message_receiver.into_stream();
        let mut buffer = vec![0u8; MSG_BUFFER_SIZE.load(Ordering::Relaxed)];

        loop {
            select! {
                num_read = recv_message(&mut substream, &mut buffer).fuse() => {
                    match num_read {
                        Err(e) => {
                            // TODO: maybe only break if e == StreamClosedByRemote
                            error!("{:?}", e);

                            if let Err(e) = internal_event_sender
                                .send_async(InternalEvent::ConnectionDropped {
                                    peer_id: peer_id.clone(),
                                    peer_address: peer_address.clone(),
                                })
                                .await
                                .map_err(|_| Error::InternalEventSendFailure("ConnectionDropped"))
                            {
                                warn!("{:?}", e);
                            }


                            // Stream to remote stopped => shut down this task
                            break;
                        }
                        Ok(num_read) => {
                            if let Err(e) = process_read(peer_id.clone(), num_read, &mut internal_event_sender, &buffer).await
                            {
                                error!("{:?}", e);
                            }
                        }
                    }
                }
                message = fused_message_receiver.next() => {
                    if let Some(message) = message {
                        if let Err(e) = send_message(&mut substream, &message).await {
                            error!("{:?}", e);
                            continue;
                        }
                    } else {
                        // Data receiver closed (due to deallocation) => shut down this task
                        break;
                    }

                }
            }
        }
    })
}

async fn send_message<S>(stream: &mut S, message: &[u8]) -> Result<(), Error>
where
    S: AsyncWrite + Unpin,
{
    stream.write_all(message).await.map_err(|_| Error::MessageSendError)?;
    stream.flush().await.map_err(|_| Error::MessageSendError)?;

    trace!("Wrote {} bytes to stream.", message.len());
    Ok(())
}

async fn recv_message<S>(stream: &mut S, message: &mut [u8]) -> Result<usize, Error>
where
    S: AsyncRead + Unpin,
{
    let num_read = stream.read(message).await.map_err(|_| Error::MessageRecvError)?;
    if num_read == 0 {
        // EOF
        debug!("Stream was closed remotely (EOF).");
        return Err(Error::StreamClosedByRemote);
    }

    trace!("Read {} bytes from stream.", num_read);
    Ok(num_read)
}

async fn process_read(
    peer_id: PeerId,
    num_read: usize,
    internal_event_sender: &mut InternalEventSender,
    buffer: &[u8],
) -> Result<(), Error> {
    // Allocate a properly sized message buffer
    let mut message = vec![0u8; num_read];
    message.copy_from_slice(&buffer[0..num_read]);

    internal_event_sender
        .send_async(InternalEvent::MessageReceived {
            message,
            from: peer_id.clone(),
        })
        .await
        .map_err(|_| Error::InternalEventSendFailure("MessageReceived"))?;

    Ok(())
}

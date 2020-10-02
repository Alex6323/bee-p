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

use crate::{
    command::Command,
    endpoint::{
        connect::ConnectedEndpointList,
        contact::{EndpointContactList, EndpointContactParams},
        EndpointId,
    },
    event::Event,
    tcp,
    util::TransportProtocol,
    RECONNECT_INTERVAL,
};

use bee_common::{shutdown::ShutdownListener, worker::Error as WorkerError};

use futures::{select, FutureExt, StreamExt};
use log::*;

use std::{sync::atomic::Ordering, time::Duration};

type CommandReceiver = flume::Receiver<Command>;
type EventReceiver = flume::Receiver<Event>;
type EventSender = flume::Sender<Event>;

pub struct EndpointWorker {
    command_receiver: flume::r#async::RecvStream<'static, Command>,
    event_sender: EventSender,
    internal_event_receiver: flume::r#async::RecvStream<'static, Event>,
    internal_event_sender: EventSender,
    endpoint_contacts: EndpointContactList,
    shutdown_listener: ShutdownListener,
}

impl EndpointWorker {
    pub async fn new(
        command_receiver: CommandReceiver,
        event_sender: EventSender,
        internal_event_receiver: EventReceiver,
        internal_event_sender: EventSender,
        endpoint_contacts: EndpointContactList,
        shutdown_listener: ShutdownListener,
    ) -> Self {
        trace!("Starting endpoint worker...");

        Self {
            command_receiver: command_receiver.into_stream(),
            event_sender,
            internal_event_receiver: internal_event_receiver.into_stream(),
            internal_event_sender,
            endpoint_contacts,
            shutdown_listener,
        }
    }

    pub async fn run(self) -> Result<(), WorkerError> {
        trace!("Endpoint worker running...");

        let EndpointWorker {
            mut command_receiver,
            mut event_sender,
            mut internal_event_receiver,
            mut internal_event_sender,
            mut endpoint_contacts,
            shutdown_listener,
            ..
        } = self;

        let mut connected_endpoints = ConnectedEndpointList::new();
        let mut fused_shutdown_listener = shutdown_listener.fuse();

        loop {
            select! {
                _ = fused_shutdown_listener => {
                    break;
                },
                command = command_receiver.next() => {
                    if !process_command(command, &mut endpoint_contacts, &mut connected_endpoints, &mut event_sender, &mut internal_event_sender).await? {
                        break;
                    }
                },
                event = internal_event_receiver.next() => {
                    if !process_event(event, &mut endpoint_contacts, &mut connected_endpoints, &mut event_sender, &mut internal_event_sender).await? {
                        break;
                    }
                },
            }
        }

        trace!("Stopped endpoint worker.");
        Ok(())
    }
}

#[inline]
async fn process_command(
    command: Option<Command>,
    mut endpoint_contacts: &mut EndpointContactList,
    mut connected_endpoints: &mut ConnectedEndpointList,
    event_sender: &mut EventSender,
    mut internal_event_sender: &mut EventSender,
) -> Result<bool, WorkerError> {
    let command = if let Some(command) = command {
        command
    } else {
        error!("Command channel unexpectedly closed.");
        return Ok(false);
    };

    trace!("Received {}.", command);

    match command {
        Command::AddEndpoint { url } => {
            add_endpoint(&url, &mut endpoint_contacts, &mut internal_event_sender).await?;
        }

        Command::RemoveEndpoint { epid } => {
            remove_endpoint(
                epid,
                &mut endpoint_contacts,
                &mut connected_endpoints,
                &mut internal_event_sender,
            )
            .await?;
        }

        Command::ConnectEndpoint { epid } => {
            connect_endpoint(
                epid,
                &mut endpoint_contacts,
                &mut connected_endpoints,
                &mut internal_event_sender,
            )
            .await?;
        }

        Command::DisconnectEndpoint { epid } => {
            if disconnect_endpoint(epid, &mut connected_endpoints)? {
                event_sender
                    .send_async(Event::EndpointDisconnected { epid })
                    .await
                    .map_err(|e| WorkerError(Box::new(e)))?;
            }
        }

        Command::SendMessage { receiver_epid, message } => {
            send_message(receiver_epid, message, &mut connected_endpoints).await?;
        }

        Command::MarkDuplicate {
            duplicate_epid,
            original_epid,
        } => {
            mark_duplicate(
                duplicate_epid,
                original_epid,
                &mut connected_endpoints,
                &mut internal_event_sender,
            )?;
        }
    }

    Ok(true)
}

#[inline]
async fn process_event(
    event: Option<Event>,
    endpoint_contacts: &mut EndpointContactList,
    connected_endpoints: &mut ConnectedEndpointList,
    event_sender: &mut EventSender,
    mut internal_event_sender: &mut EventSender,
) -> Result<bool, WorkerError> {
    let event = if let Some(event) = event {
        event
    } else {
        error!("Event channel unexpectedly closed.");
        return Ok(false);
    };

    trace!("Received {}.", event);

    match event {
        Event::EndpointAdded { epid } => {
            event_sender
                .send_async(Event::EndpointAdded { epid })
                .await
                .map_err(|e| WorkerError(Box::new(e)))?;
        }

        Event::EndpointRemoved { epid } => {
            event_sender
                .send_async(Event::EndpointRemoved { epid })
                .await
                .map_err(|e| WorkerError(Box::new(e)))?;
        }

        Event::ConnectionEstablished {
            epid,
            peer_address,
            origin,
            data_sender,
        } => {
            connected_endpoints.insert(epid, data_sender);

            event_sender
                .send_async(Event::EndpointConnected {
                    epid,
                    peer_address,
                    origin,
                })
                .await
                .map_err(|e| WorkerError(Box::new(e)))?
        }

        Event::ConnectionDropped { epid } => {
            // NOTE: we allow duplicates to be disconnected (no reconnect)
            if connected_endpoints.is_duplicate(epid) {
                if connected_endpoints.remove(epid) {
                    event_sender
                        .send_async(Event::EndpointDisconnected { epid })
                        .await
                        .map_err(|e| WorkerError(Box::new(e)))?;
                } else {
                    warn!("ConnectionDropped fired, but endpoint was already unregistered.");
                }
                return Ok(true);
            }

            // NOTE: we allow originals to be disconnected (no reconnect), if there's a duplicate
            if connected_endpoints.has_duplicate(epid).is_some() {
                warn!("A connection was dropped that still has a connected duplicate."); // Should we also disconnect the duplicate?
                if connected_endpoints.remove(epid) {
                    event_sender
                        .send_async(Event::EndpointDisconnected { epid })
                        .await
                        .map_err(|e| WorkerError(Box::new(e)))?;
                } else {
                    warn!("ConnectionDropped fired, but endpoint was already unregistered.");
                }
                return Ok(true);
            }

            // TODO: check, if the contact belonging to the dropped connection is still a "wanted" peer
            if endpoint_contacts.contains(epid) {
                connect_endpoint(epid, endpoint_contacts, connected_endpoints, &mut internal_event_sender).await?;
            }
        }

        Event::MessageReceived { epid, message } => event_sender
            .send_async(Event::MessageReceived { epid, message })
            .await
            .map_err(|e| WorkerError(Box::new(e)))?,

        Event::ReconnectTimerElapsed { epid } => {
            connect_endpoint(epid, endpoint_contacts, connected_endpoints, &mut internal_event_sender).await?;
        }
        _ => (),
    }

    Ok(true)
}
#[inline]
async fn add_endpoint(
    url: &str,
    endpoint_contacts: &mut EndpointContactList,
    internal_event_sender: &mut EventSender,
) -> Result<bool, WorkerError> {
    if let Ok(endpoint_params) = EndpointContactParams::from_url(url.clone()).await {
        // let epid = EndpointId::new();
        let epid = endpoint_params.create_epid();

        if endpoint_contacts.insert(epid, endpoint_params) {
            internal_event_sender
                .send_async(Event::EndpointAdded { epid })
                .await
                .map_err(|e| WorkerError(Box::new(e)))?;

            Ok(true)
        } else {
            Ok(false)
        }
    } else {
        Ok(false)
    }
}

#[inline]
async fn remove_endpoint(
    epid: EndpointId,
    endpoint_contacts: &mut EndpointContactList,
    connected_endpoints: &mut ConnectedEndpointList,
    internal_event_sender: &mut EventSender,
) -> Result<bool, WorkerError> {
    if endpoint_contacts.remove(epid) {
        if connected_endpoints.remove(epid) {
            trace!("Removed and disconnected endpoint {}.", epid);
        } else {
            trace!("Removed endpoint {}.", epid);
        }

        internal_event_sender
            .send_async(Event::EndpointRemoved { epid })
            .await
            .map_err(|e| WorkerError(Box::new(e)))?;

        Ok(true)
    } else {
        Ok(false)
    }
}

#[inline]
async fn connect_endpoint(
    epid: EndpointId,
    endpoint_contacts: &mut EndpointContactList,
    connected_endpoints: &mut ConnectedEndpointList,
    internal_event_sender: &mut EventSender,
) -> Result<bool, WorkerError> {
    // NOTE: 'unwrap' is safe, because we assume this method is only called correctly after making sure the endpoint is
    // still part of the contact-list.
    let mut endpoint_params = endpoint_contacts.get(epid).unwrap();
    if connected_endpoints.contains(epid) {
        // NOTE: already connected
        return Ok(false);
    }
    match endpoint_params.transport_protocol {
        TransportProtocol::Tcp => {
            // NOTE: 'unwrap' here, because the cache should never be empty at this point.
            // TODO: impl refresh (make sure to not operate on the clone)
            let socket_address = endpoint_params.socket_address(false).await.unwrap();

            if tcp::connect_endpoint(epid, socket_address, internal_event_sender.clone())
                .await
                .is_ok()
            {
                Ok(true)
            } else {
                tokio::spawn(send_event_after_delay(
                    Event::ReconnectTimerElapsed { epid },
                    internal_event_sender.clone(),
                ));
                Ok(false)
            }
        }
        TransportProtocol::Udp => unimplemented!("Support for UDP endpoints is not yet implemented"),
    }
}

#[inline]
fn disconnect_endpoint(epid: EndpointId, connected_endpoints: &mut ConnectedEndpointList) -> Result<bool, WorkerError> {
    // NOTE: removing the endpoint will drop the connection!
    Ok(connected_endpoints.remove(epid))
}

#[inline]
async fn send_event_after_delay(event: Event, internal_event_sender: EventSender) -> Result<(), WorkerError> {
    tokio::time::delay_for(Duration::from_secs(RECONNECT_INTERVAL.load(Ordering::Relaxed))).await;

    Ok(internal_event_sender
        .send_async(event)
        .await
        .map_err(|e| WorkerError(Box::new(e)))?)
}

#[inline]
async fn send_message(
    receiver_epid: EndpointId,
    message: Vec<u8>,
    connected: &mut ConnectedEndpointList,
) -> Result<bool, WorkerError> {
    Ok(connected.send_message(message, receiver_epid).await?)
}

#[inline]
fn mark_duplicate(
    duplicate_epid: EndpointId,
    original_epid: EndpointId,
    connected_endpoints: &mut ConnectedEndpointList,
    _internal_event_sender: &mut EventSender,
) -> Result<bool, WorkerError> {
    Ok(connected_endpoints.mark_duplicate(duplicate_epid, original_epid))
}

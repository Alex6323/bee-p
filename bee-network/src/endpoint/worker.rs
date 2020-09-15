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
    commands::Command,
    endpoint::{
        connect::ConnectedEndpointList,
        contact::{EndpointContactList, EndpointContactParams},
        EndpointId,
    },
    events::Event,
    tcp,
    util::TransportProtocol,
    RECONNECT_INTERVAL,
};

use bee_common::{shutdown::ShutdownListener, worker::Error as WorkerError};

use futures::{channel::mpsc, select, sink::SinkExt, stream, FutureExt, StreamExt};
use log::*;

use std::{sync::atomic::Ordering, time::Duration};

type CommandReceiver = mpsc::UnboundedReceiver<Command>;
type EventReceiver = mpsc::UnboundedReceiver<Event>;
type EventSender = mpsc::UnboundedSender<Event>;

pub struct EndpointWorker {
    command_receiver: stream::Fuse<CommandReceiver>,
    event_sender: EventSender,
    internal_event_receiver: stream::Fuse<EventReceiver>,
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
        debug!("Starting endpoint worker...");

        Self {
            command_receiver: command_receiver.fuse(),
            event_sender,
            internal_event_receiver: internal_event_receiver.fuse(),
            internal_event_sender,
            endpoint_contacts,
            shutdown_listener,
        }
    }

    pub async fn run(self) -> Result<(), WorkerError> {
        let EndpointWorker {
            mut command_receiver,
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
                    if !process_command(command, &mut endpoint_contacts, &mut connected_endpoints, &mut internal_event_sender).await? {
                        break;
                    }
                },
                event = internal_event_receiver.next() => {
                    if !process_event(event, &mut endpoint_contacts, &mut connected_endpoints).await? {
                        break;
                    }
                },
            }
        }

        debug!("Stopped endpoint worker.");
        Ok(())
    }
}

#[inline]
async fn process_command(
    command: Option<Command>,
    mut endpoint_contacts: &mut EndpointContactList,
    mut connected_endpoints: &mut ConnectedEndpointList,
    mut internal_event_sender: &mut EventSender,
) -> Result<bool, WorkerError> {
    let command = if let Some(command) = command {
        command
    } else {
        error!("Command channel unexpectedly closed.");
        return Ok(false);
    };

    debug!("Received {}.", command);

    match command {
        Command::AddEndpoint { url } => {
            add_endpoint(&url, &mut endpoint_contacts, &mut internal_event_sender).await?;
        }

        Command::RemoveEndpoint { epid } => {
            remove_endpoint(
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
                self.event_sender.send(Event::EndpointDisconnected { epid }).await?;
            }
        }

        Command::SendMessage { epid, message } => {
            send_message(&epid, message, &mut connected_endpoints).await?;
        }

        Command::SetDuplicate { epid, of } => {
            set_duplicate(epid, of, &mut connected_endpoints, &mut self.internal_event_sender)?;
        }
    }

    Ok(true)
}

#[inline]
async fn process_event(
    event: Option<Event>,
    endpoint_contacts: &mut EndpointContactList,
    connected_endpoints: &mut ConnectedEndpointList,
) -> Result<bool, WorkerError> {
    let event = if let Some(event) = event {
        event
    } else {
        error!("Event channel unexpectedly closed.");
        return Ok(false);
    };

    debug!("Received {}.", event);

    match event {
        Event::EndpointAdded { epid } => {
            self.event_sender.send(Event::EndpointAdded { epid }).await?;
        }

        Event::EndpointRemoved { epid } => {
            self.event_sender.send(Event::EndpointRemoved { epid }).await?;
        }

        Event::ConnectionEstablished {
            epid,
            socket_address,
            origin,
            sender,
        } => {
            connected_endpoints.insert(epid, socket_address, sender);

            self.event_sender
                .send(Event::EndpointConnected {
                    epid,
                    socket_address,
                    origin,
                })
                .await?
        }

        Event::ConnectionDropped { epid } => {
            // NOTE: we allow duplicates to be disconnected (no reconnect)
            if connected_endpoints.is_duplicate(&epid) {
                if connected_endpoints.remove(&epid) {
                    self.event_sender.send(Event::EndpointDisconnected { epid }).await?;
                } else {
                    warn!("ConnectionDropped fired, but endpoint was already removed from list");
                }
                return Ok(true);
            }

            // NOTE: we allow originals to be disconnected (no reconnect), if there's a duplicate
            if connected_endpoints.has_duplicate(&epid).is_some() {
                warn!("A connection was dropped that still has a connected duplicate."); // Should we also disconnect the duplicate?
                if connected_endpoints.remove(&epid) {
                    self.event_sender.send(Event::EndpointDisconnected { epid }).await?;
                } else {
                    warn!("ConnectionDropped fired, but endpoint was already removed from list");
                }
                return Ok(true);
            }

            // TODO: check, if the contact belonging to the dropped connection is still a "wanted" peer
            //
            //
            //

            connect_endpoint(
                epid,
                endpoint_contacts,
                connected_endpoints,
                &mut self.internal_event_sender,
            )
            .await?;
        }

        Event::MessageReceived { epid, message } => {
            self.event_sender.send(Event::MessageReceived { epid, message }).await?
        }

        Event::TimerElapsed { epid } => {
            connect_endpoint(
                epid,
                endpoint_contacts,
                connected_endpoints,
                &mut self.internal_event_sender,
            )
            .await?;
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
    if let Ok(contact_params) = EndpointContactParams::from_url(url.clone()) {
        let epid = contact_params.epid;

        if endpoint_contacts.insert(endpoint) {
            // Add to allowlist
            let allowlist = allowlist::get();
            allowlist.insert(epid, url);

            internal_event_sender.send(Event::EndpointAdded { epid }).await?;

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
    if endpoint_contacts.remove(url) {
        if connected_endpoints.remove(&epid) {
            debug!("Removed connected endpoint {}.", epid);
        } else {
            debug!("Removed unconnected endpoint {}.", epid);
        }

        // Remove from allowlist
        let allowlist = allowlist::get();
        allowlist.remove(&epid);

        internal_event_sender.send(Event::EndpointRemoved { epid }).await?;
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
    if let Some(endpoint) = endpoint_contacts.get_mut(&epid) {
        if connected_endpoints.contains(&endpoint.epid) {
            Ok(false)
        } else {
            match endpoint.protocol {
                TransportProtocol::Tcp => {
                    let epid = &endpoint.epid;
                    let address = &endpoint.address;

                    if tcp::client::connect_endpoint(epid, address, internal_event_sender.clone())
                        .await
                        .is_ok()
                    {
                        // connected_endpoints.insert(*epid, *address, timestamp, bytes_sender);
                        Ok(true)
                    } else {
                        tokio::spawn(send_event_after_delay(
                            Event::TimerElapsed { epid },
                            internal_event_sender.clone(),
                        ));
                        Ok(false)
                    }
                }
                TransportProtocol::Udp => unimplemented!("Support for UDP endpoints is not yet implemented"),
            }
        }
    } else {
        Ok(false)
    }
}

#[inline]
fn disconnect_endpoint(epid: EndpointId, connected_endpoints: &mut ConnectedEndpointList) -> Result<bool, WorkerError> {
    // NOTE: removing the endpoint will drop the connection!
    Ok(connected_endpoints.remove(&epid))
}

#[inline]
async fn send_event_after_delay(event: Event, mut internal_event_sender: EventSender) -> Result<(), WorkerError> {
    tokio::time::delay_for(Duration::from_secs(RECONNECT_INTERVAL.load(Ordering::Relaxed))).await;

    Ok(internal_event_sender.send(event).await?)
}

#[inline]
async fn send_message(
    epid: &EndpointId,
    message: Vec<u8>,
    connected: &mut ConnectedEndpointList,
) -> Result<bool, WorkerError> {
    Ok(connected.send(message, epid).await?)
}

#[inline]
fn set_duplicate(
    epid: EndpointId,
    of: EndpointId,
    connected_endpoints: &mut ConnectedEndpointList,
    internal_event_sender: &mut EventSender,
) -> Result<bool, WorkerError> {
    Ok(connected_endpoints.set_duplicate(epid, of))
}

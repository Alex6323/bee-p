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

use crate::util::net::IpFilter;

use crate::{
    commands::Command,
    endpoint::{
        connect::ConnectedEndpointList,
        contact::{EndpointContactList, EndpointContactParams},
        EndpointId,
    },
    events::Event,
    tcp, RECONNECT_INTERVAL,
};

use bee_common::{shutdown::ShutdownListener, worker::Error as WorkerError};

use futures::{channel::mpsc, select, sink::SinkExt, stream, FutureExt, StreamExt};
use log::*;

use std::{sync::atomic::Ordering, time::Duration};

type CommandReceiver = mpsc::Receiver<Command>;
type EventReceiver = mpsc::Receiver<Event>;
type EventSender = mpsc::Sender<Event>;

pub struct EndpointWorker {
    command_receiver: stream::Fuse<CommandReceiver>,
    event_sender: EventSender,
    internal_event_receiver: stream::Fuse<EventReceiver>,
    internal_event_sender: EventSender,
    ip_filter: IpFilter,
}

impl EndpointWorker {
    pub fn new(
        command_receiver: CommandReceiver,
        event_sender: EventSender,
        internal_event_receiver: EventReceiver,
        internal_event_sender: EventSender,
        ip_filter: IpFilter,
    ) -> Self {
        Self {
            command_receiver: command_receiver.fuse(),
            event_sender,
            internal_event_receiver: internal_event_receiver.fuse(),
            internal_event_sender,
            ip_filter,
        }
    }

    pub async fn run(mut self, shutdown_listener: ShutdownListener) -> Result<(), WorkerError> {
        debug!("Starting endpoint worker...");

        let mut endpoint_contacts = EndpointContactList::new();
        let mut connected_endpoints = ConnectedEndpointList::new();

        let mut fused_shutdown_listener = shutdown_listener.fuse();

        loop {
            select! {
                _ = fused_shutdown_listener => {
                    break;
                },
                command = self.command_receiver.next() => {
                    if !self.handle_command(command, &mut endpoint_contacts, &mut connected_endpoints).await? {
                        break;
                    }
                },
                event = self.internal_event_receiver.next() => {
                    if !self.handle_event(event, &mut endpoint_contacts, &mut connected_endpoints).await? {
                        break;
                    }
                },
            }
        }

        debug!("Stopped endpoint worker.");
        Ok(())
    }

    #[inline]
    async fn handle_command(
        &mut self,
        command: Option<Command>,
        mut endpoint_contacts: &mut EndpointContactList,
        mut connected_endpoints: &mut ConnectedEndpointList,
    ) -> Result<bool, WorkerError> {
        let command = if let Some(command) = command {
            command
        } else {
            error!("Command channel unexpectedly closed.");
            return Ok(false);
        };

        debug!("Received {}.", command);

        match command {
            Command::AddContact { url } => {
                add_contact(&url, &mut endpoint_contacts, &mut self.internal_event_sender).await?;
            }

            Command::RemoveContact { url } => {
                remove_contact(
                    &url,
                    &mut endpoint_contacts,
                    &mut connected_endpoints,
                    &mut self.internal_event_sender,
                )
                .await?;
            }

            Command::ConnectEndpoint { epid } => {
                try_connect_to(
                    epid,
                    &mut contacts,
                    &mut connected_endpoints,
                    &mut self.internal_event_sender,
                )
                .await?;
            }

            Command::DisconnectEndpoint { epid } => {
                if disconnect(epid, &mut connected_endpoints) {
                    self.event_sender
                        .send(Event::EndpointDisconnected {
                            epid,
                            total: connected_endpoints.len(),
                        })
                        .await?;
                }
            }

            Command::SendMessage { epid, message } => {
                send_message(&epid, message, &mut connected_endpoints).await?;
            }

            Command::SetDuplicate {
                duplicate,
                duplicate_of,
            } => {
                set_duplicate(
                    duplicate,
                    duplicate_of,
                    &mut connected_endpoints,
                    &mut self.internal_event_sender,
                )?;
            }
        }

        Ok(true)
    }

    #[inline]
    async fn handle_event(
        &mut self,
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
            Event::EndpointAdded { epid, total } => {
                self.event_sender.send(Event::EndpointAdded { epid, total }).await?;
            }

            Event::EndpointRemoved { epid, total } => {
                self.event_sender.send(Event::EndpointRemoved { epid, total }).await?;
            }

            Event::ConnectionCreated {
                endpoint,
                origin,
                data_sender,
                timestamp,
            } => {
                let epid = endpoint.epid;
                let address = endpoint.address;

                connected_endpoints.insert(epid, address, timestamp, data_sender);

                self.event_sender
                    .send(Event::EndpointConnected {
                        epid,
                        address,
                        origin,
                        total: connected_endpoints.len(),
                    })
                    .await?
            }

            Event::ConnectionDropped { epid } => {
                let is_disconnected = disconnect(epid, connected_endpoints);

                if is_disconnected {
                    self.event_sender
                        .send(Event::EndpointDisconnected {
                            epid,
                            total: connected_endpoints.len(),
                        })
                        .await?;
                }

                if !connected_endpoints.is_duplicate(&epid) && !connected_endpoints.has_duplicate(&epid) {
                    try_connect_to(epid, endpoints, connected_endpoints, &mut self.internal_event_sender).await?;
                }
            }

            Event::MessageReceived { epid, message } => {
                self.event_sender.send(Event::MessageReceived { epid, message }).await?
            }

            Event::TryConnectTo { epid } => {
                try_connect_to(epid, endpoints, connected_endpoints, &mut self.internal_event_sender).await?;
            }
            _ => (),
        }

        Ok(true)
    }
}

#[inline]
async fn add_contact(
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

            internal_event_sender
                .send(Event::EndpointAdded {
                    epid,
                    total: endpoints.len(),
                })
                .await?;

            Ok(true)
        } else {
            Ok(false)
        }
    } else {
        Ok(false)
    }
}

#[inline]
async fn remove_contact(
    url: &str,
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

        internal_event_sender
            .send(Event::EndpointRemoved {
                epid,
                total: endpoints.len(),
            })
            .await?;
        Ok(true)
    } else {
        Ok(false)
    }
}

#[inline]
async fn try_connect_to(
    epid: EndpointId,
    endpoints: &mut EndpointList,
    connected_endpoints: &mut ConnectedEndpointList,
    internal_event_sender: &mut EventSender,
) -> Result<bool, WorkerError> {
    if let Some(endpoint) = endpoints.get_mut(&epid) {
        if connected_endpoints.contains(&endpoint.epid) {
            Ok(false)
        } else {
            match endpoint.protocol {
                Protocol::Tcp => {
                    let epid = &endpoint.epid;
                    let address = &endpoint.address;

                    if tcp::try_connect_to(epid, address, internal_event_sender.clone())
                        .await
                        .is_ok()
                    {
                        // connected_endpoints.insert(*epid, *address, timestamp, bytes_sender);
                        Ok(true)
                    } else {
                        tokio::spawn(send_event_after_delay(
                            Event::TryConnectTo { epid: *epid },
                            internal_event_sender.clone(),
                        ));
                        Ok(false)
                    }
                }
                Protocol::Udp => unimplemented!("Support for UDP endpoints is not yet implemented"),
            }
        }
    } else {
        Ok(false)
    }
}

#[inline]
async fn send_event_after_delay(event: Event, mut internal_event_sender: EventSender) -> Result<(), WorkerError> {
    tokio::time::delay_for(Duration::from_secs(RECONNECT_INTERVAL.load(Ordering::Relaxed))).await;

    Ok(internal_event_sender.send(event).await?)
}

#[inline]
fn disconnect(epid: EndpointId, connected: &mut ConnectedEndpointList) -> bool {
    connected.remove(&epid)
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
    duplicate: EndpointId,
    duplicate_of: EndpointId,
    connected_endpoints: &mut ConnectedEndpointList,
    internal_event_sender: &mut EventSender,
) -> Result<bool, WorkerError> {
    Ok(connected_endpoints.set_duplicate(duplicate, duplicate_of))
}

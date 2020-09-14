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

use super::allowlist;

use crate::{
    commands::Command,
    endpoint::{connected::ConnectedEndpointList, Endpoint, EndpointId, EndpointList},
    events::Event,
    tcp, RECONNECT_INTERVAL,
};

use bee_common::{shutdown::ShutdownListener, worker::Error as WorkerError};

use futures::{channel::mpsc, select, sink::SinkExt, stream, FutureExt, StreamExt};
use log::*;
use tokio::task;

use std::{sync::atomic::Ordering, time::Duration};

type CommandReceiver = mpsc::Receiver<Command>;
type EventReceiver = mpsc::Receiver<Event>;
type EventSender = mpsc::Sender<Event>;

pub struct EndpointWorker {
    command_receiver: stream::Fuse<CommandReceiver>,
    event_sender: EventSender,
    internal_event_receiver: stream::Fuse<EventReceiver>,
    internal_event_sender: EventSender,
}

impl EndpointWorker {
    pub fn new(
        command_receiver: CommandReceiver,
        event_sender: EventSender,
        internal_event_receiver: EventReceiver,
        internal_event_sender: EventSender,
    ) -> Self {
        Self {
            command_receiver: command_receiver.fuse(),
            event_sender,
            internal_event_receiver: internal_event_receiver.fuse(),
            internal_event_sender,
        }
    }

    pub async fn run(mut self, shutdown_listener: ShutdownListener) -> Result<(), WorkerError> {
        debug!("Starting endpoint worker...");

        let mut endpoints = EndpointList::new();
        let mut connected_endpoints = ConnectedEndpointList::new();

        let mut fused_shutdown_listener = shutdown_listener.fuse();

        loop {
            select! {
                _ = fused_shutdown_listener => {
                    break;
                },
                command = self.command_receiver.next() => {
                    if !self.handle_command(command, &mut endpoints, &mut connected_endpoints).await? {
                        break;
                    }
                },
                event = self.internal_event_receiver.next() => {
                    if !self.handle_event(event, &mut endpoints, &mut connected_endpoints).await? {
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
        mut endpoints: &mut EndpointList,
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
            Command::AddEndpoint { url } => {
                add_endpoint(&mut endpoints, url, &mut self.internal_event_sender).await?;
            }

            Command::RemoveEndpoint { epid } => {
                remove_endpoint(
                    epid,
                    &mut endpoints,
                    &mut connected_endpoints,
                    &mut self.internal_event_sender,
                )
                .await?;
            }

            Command::Connect { epid } => {
                try_connect_to(
                    epid,
                    &mut endpoints,
                    &mut connected_endpoints,
                    &mut self.internal_event_sender,
                )
                .await?;
            }

            Command::Disconnect { epid } => {
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

            Command::SetDuplicate { epid, other } => {
                remove_duplicate(epid, other, &mut connected_endpoints, &mut self.internal_event_sender).await?;
            }
        }

        Ok(true)
    }

    #[inline]
    async fn handle_event(
        &mut self,
        event: Option<Event>,
        endpoints: &mut EndpointList,
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
async fn add_endpoint(
    endpoints: &mut EndpointList,
    url: Url,
    internal_event_sender: &mut EventSender,
) -> Result<bool, WorkerError> {
    let endpoint = Endpoint::from_url(url.clone()).await;
    let epid = endpoint.epid;

    if endpoints.insert(endpoint) {
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
}

#[inline]
async fn remove_endpoint(
    epid: EndpointId,
    endpoints: &mut EndpointList,
    connected_endpoints: &mut ConnectedEndpointList,
    internal_event_sender: &mut EventSender,
) -> Result<bool, WorkerError> {
    if endpoints.remove(&epid) {
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
async fn remove_duplicate(
    epid: EndpointId,
    other: EndpointId,
    connected_endpoints: &mut ConnectedEndpointList,
    internal_event_sender: &mut EventSender,
) -> Result<(), WorkerError> {
    if connected_endpoints.set_duplicate(epid, other) && connected_endpoints.remove_duplicate(&epid, &other) {
        internal_event_sender.send(Event::ConnectionDropped { epid }).await?;
    }
    Ok(())
}

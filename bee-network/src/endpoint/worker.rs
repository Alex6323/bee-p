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
    address::url::{Protocol, Url},
    commands::Command,
    endpoint::{outbox::Outbox, store::Endpoints, Endpoint as Ep, EndpointId as EpId},
    events::Event,
    tcp,
    utils::time,
};

use bee_common::{shutdown::ShutdownListener, worker::Error as WorkerError};

use async_std::task::{self, spawn};
use futures::{channel::mpsc, select, sink::SinkExt, stream, FutureExt, StreamExt};
use log::*;

use std::time::Duration;

type CommandReceiver = mpsc::Receiver<Command>;
type EventReceiver = mpsc::Receiver<Event>;
type EventSender = mpsc::Sender<Event>;

pub struct EndpointWorker {
    command_receiver: stream::Fuse<CommandReceiver>,
    event_sender: EventSender,
    internal_event_receiver: stream::Fuse<EventReceiver>,
    internal_event_sender: EventSender,
    reconnect_interval: Duration,
}

impl EndpointWorker {
    pub fn new(
        command_receiver: CommandReceiver,
        event_sender: EventSender,
        internal_event_receiver: EventReceiver,
        internal_event_sender: EventSender,
        reconnect_interval: Duration,
    ) -> Self {
        Self {
            command_receiver: command_receiver.fuse(),
            event_sender,
            internal_event_receiver: internal_event_receiver.fuse(),
            internal_event_sender,
            reconnect_interval,
        }
    }

    pub async fn run(mut self, shutdown_listener: ShutdownListener) -> Result<(), WorkerError> {
        debug!("Starting endpoint worker...");

        let mut contacts = Endpoints::new();

        // TODO: those two probably need to be merged as each connected endpoint is also part of the outbox
        let mut connected = Endpoints::new();
        let mut outbox = Outbox::new();

        let mut fused_shutdown_listener = shutdown_listener.fuse();

        loop {
            select! {
                _ = fused_shutdown_listener => {
                    break;
                },
                command = self.command_receiver.next() => {
                    if !self.handle_command(command, &mut contacts, &mut connected, &mut outbox).await? {
                        break;
                    }
                },
                event = self.internal_event_receiver.next() => {
                    if !self.handle_event(event, &mut contacts, &mut connected, &mut outbox).await? {
                        break;
                    }
                }
            }
        }

        debug!("Stopped endpoint worker.");
        Ok(())
    }

    #[inline]
    async fn handle_command(
        &mut self,
        command: Option<Command>,
        mut contacts: &mut Endpoints,
        mut connected: &mut Endpoints,
        mut outbox: &mut Outbox,
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
                add_endpoint(&mut contacts, url, &mut self.internal_event_sender).await?;
            }
            Command::RemoveEndpoint { epid } => {
                remove_endpoint(
                    epid,
                    &mut contacts,
                    &mut connected,
                    &mut outbox,
                    &mut self.internal_event_sender,
                )
                .await?;
            }
            Command::Connect { epid } => {
                try_connect(
                    epid,
                    self.reconnect_interval,
                    &mut contacts,
                    &mut connected,
                    &mut self.internal_event_sender,
                )
                .await?;
            }
            Command::Disconnect { epid } => {
                if disconnect(epid, &mut connected, &mut outbox).await {
                    self.event_sender
                        .send(Event::EndpointDisconnected {
                            epid,
                            total: connected.num(),
                        })
                        .await?;
                }
            }
            Command::SendMessage { epid, bytes } => {
                send_bytes(&epid, bytes, &mut outbox).await?;
            }
        }

        Ok(true)
    }

    #[inline]
    async fn handle_event(
        &mut self,
        event: Option<Event>,
        contacts: &mut Endpoints,
        connected: &mut Endpoints,
        outbox: &mut Outbox,
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
            Event::NewConnection {
                endpoint,
                origin,
                sender,
            } => {
                let epid = endpoint.id;
                let addr = endpoint.address;

                outbox.insert(epid, sender);
                connected.insert(endpoint);

                self.event_sender
                    .send(Event::EndpointConnected {
                        epid,
                        address: addr,
                        origin,
                        timestamp: time::timestamp_millis(),
                        total: connected.num(),
                    })
                    .await?
            }
            Event::LostConnection { epid } => {
                let is_disconnected = disconnect(epid, connected, outbox).await;

                if is_disconnected {
                    self.event_sender
                        .send(Event::EndpointDisconnected {
                            epid,
                            total: connected.num(),
                        })
                        .await?;
                }

                // TODO: do not try to reconnect to duplicate endpoints
                // NOTE: 'try_connect' will check if 'epid' is part of the contact list
                try_connect(
                    epid,
                    self.reconnect_interval,
                    contacts,
                    connected,
                    &mut self.internal_event_sender,
                )
                .await?;
            }
            Event::MessageSent { epid, num_bytes } => {
                self.event_sender.send(Event::MessageSent { epid, num_bytes }).await?
            }
            Event::MessageReceived { epid, bytes } => {
                self.event_sender.send(Event::MessageReceived { epid, bytes }).await?
            }
            Event::TryConnect { epid } => {
                try_connect(
                    epid,
                    self.reconnect_interval,
                    contacts,
                    connected,
                    &mut self.internal_event_sender,
                )
                .await?;
            }
            _ => (),
        }

        Ok(true)
    }
}

#[inline]
async fn add_endpoint(
    contacts: &mut Endpoints,
    url: Url,
    internal_event_sender: &mut EventSender,
) -> Result<bool, WorkerError> {
    let ep = Ep::from_url(url);
    let epid = ep.id;

    if contacts.insert(ep) {
        // add its ip to the allowlist, so that we can make sure that we accept only connections
        // from known peers
        let allowlist = allowlist::get();
        allowlist.insert(epid, url.address().ip());

        internal_event_sender
            .send(Event::EndpointAdded {
                epid,
                total: contacts.num(),
            })
            .await?;
        Ok(true)
    } else {
        Ok(false)
    }
}

#[inline]
async fn remove_endpoint(
    epid: EpId,
    contacts: &mut Endpoints,
    connected: &mut Endpoints,
    outbox: &mut Outbox,
    internal_event_sender: &mut EventSender,
) -> Result<bool, WorkerError> {
    // NOTE: current default behavior is to drop connections once the contact is removed
    let removed_recipient = outbox.remove(&epid);
    let removed_contact = contacts.remove(&epid);
    let removed_connected = connected.remove(&epid);

    if removed_connected && !removed_recipient {
        warn!("Removed an endpoint that was connected, but couldn't be sent to.");
    }

    if removed_contact || removed_connected {
        // Remove its IP also from the allowlist, so we won't accept connections from it
        // anymore
        let allowlist = allowlist::get();
        allowlist.remove(&epid);

        internal_event_sender
            .send(Event::EndpointRemoved {
                epid,
                total: contacts.num(),
            })
            .await?;
        Ok(true)
    } else {
        Ok(false)
    }
}

#[inline]
async fn try_connect(
    epid: EpId,
    reconnect_interval: Duration,
    contacts: &mut Endpoints,
    connected: &mut Endpoints,
    internal_event_sender: &mut EventSender,
) -> Result<bool, WorkerError> {
    if let Some(ep) = contacts.get_mut(&epid) {
        if connected.contains(&ep.id) {
            Ok(false)
        } else {
            match ep.protocol {
                Protocol::Tcp => {
                    if tcp::try_connect(&ep.id, &ep.address, internal_event_sender.clone())
                        .await
                        .is_ok()
                    {
                        connected.insert(ep.clone());
                        Ok(true)
                    } else {
                        // If connection attempt fails, issue a `TryConnect` event after a certain delay.
                        // NOTE: It won't be raised, if the endpoint has been removed in the mean time.
                        spawn(raise_event_after_delay(
                            Event::TryConnect { epid },
                            reconnect_interval,
                            internal_event_sender.clone(),
                        ));
                        Ok(false)
                    }
                }
                Protocol::Udp => Ok(true),
            }
        }
    } else {
        Ok(false)
    }
}

#[inline]
async fn raise_event_after_delay(
    event: Event,
    delay: Duration,
    mut internal_event_sender: EventSender,
) -> Result<(), WorkerError> {
    task::sleep(delay).await;

    Ok(internal_event_sender.send(event).await?)
}

#[inline]
async fn disconnect(epid: EpId, connected: &mut Endpoints, outbox: &mut Outbox) -> bool {
    let removed_recipient = outbox.remove(&epid);
    let removed_connected = connected.remove(&epid);

    if removed_connected && !removed_recipient {
        warn!("Removed an endpoint that was connected, but couldn't be sent to.");
    }

    removed_connected
}

#[inline]
async fn send_bytes(recipient: &EpId, bytes: Vec<u8>, outbox: &mut Outbox) -> Result<bool, WorkerError> {
    Ok(outbox.send(bytes, recipient).await?)
}

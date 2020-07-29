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
    commands::{Command, CommandReceiver, Responder},
    endpoint::{outbox::Outbox, store::Endpoints, Endpoint as Ep, EndpointId as EpId},
    events::{Event, EventPublisher, EventSubscriber},
    tcp,
    utils::time,
};

use bee_common::{shutdown::ShutdownListener, worker::Error as WorkerError};

use async_std::task::{self, spawn};
use futures::{select, sink::SinkExt, stream, FutureExt, StreamExt};
use log::*;

use std::time::Duration;

pub struct EndpointWorker {
    fused_command_rx: stream::Fuse<CommandReceiver>,
    fused_event_sub: stream::Fuse<EventSubscriber>,
    event_pub_intern: EventPublisher,
    event_pub: EventPublisher,
    reconnect_interval: Duration,
}

impl EndpointWorker {
    pub fn new(
        command_rx: CommandReceiver,
        event_sub_intern: EventSubscriber,
        event_pub_intern: EventPublisher,
        event_pub: EventPublisher,
        reconnect_interval: Duration,
    ) -> Self {
        Self {
            fused_command_rx: command_rx.fuse(),
            fused_event_sub: event_sub_intern.fuse(),
            event_pub_intern,
            event_pub,
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
                command = self.fused_command_rx.next() => {
                    if !self.handle_command(command, &mut contacts, &mut connected, &mut outbox).await? {
                        break;
                    }
                },
                event = self.fused_event_sub.next() => {
                    if !self.handle_event(event, &mut contacts, &mut connected, &mut outbox).await? {
                        break;
                    }
                },
                _ = fused_shutdown_listener => {
                    break;
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
            Command::AddEndpoint { url, responder } => {
                let res = add_endpoint(&mut contacts, url, &mut self.event_pub_intern).await?;

                if let Some(responder) = responder {
                    if responder.send(res).is_err() {
                        warn!("Error sending command response.");
                    };
                }
            }
            Command::RemoveEndpoint { epid, responder } => {
                let res = remove_endpoint(
                    epid,
                    &mut contacts,
                    &mut connected,
                    &mut outbox,
                    &mut self.event_pub_intern,
                )
                .await?;

                if let Some(responder) = responder {
                    if responder.send(res).is_err() {
                        warn!("Error sending command response.");
                    };
                }
            }
            Command::Connect { epid, responder } => {
                try_connect(
                    epid,
                    self.reconnect_interval,
                    &mut contacts,
                    &mut connected,
                    responder,
                    &mut self.event_pub_intern,
                )
                .await?;
            }
            Command::Disconnect { epid, responder } => {
                let is_disconnected = disconnect(epid, &mut connected, &mut outbox).await;

                if let Some(responder) = responder {
                    if responder.send(is_disconnected).is_err() {
                        warn!("Error sending command response.");
                    };
                }

                if is_disconnected {
                    self.event_pub
                        .send(Event::EndpointDisconnected {
                            epid,
                            total: connected.num(),
                        })
                        .await?;
                }
            }
            Command::SendMessage { epid, bytes, responder } => {
                let res = send_bytes(&epid, bytes, &mut outbox).await?;

                if let Some(responder) = responder {
                    if responder.send(res).is_err() {
                        warn!("Error sending command response.");
                    };
                }
            }
            Command::MulticastMessage {
                epids,
                bytes,
                responder,
            } => {
                let res = multicast_bytes(&epids, bytes, &mut outbox).await?;

                if let Some(responder) = responder {
                    if responder.send(res).is_err() {
                        warn!("Error sending command response.");
                    };
                }
            }
            Command::BroadcastMessage { bytes, responder } => {
                let res = broadcast_bytes(bytes, &mut outbox).await?;

                if let Some(responder) = responder {
                    if responder.send(res).is_err() {
                        warn!("Error sending command response.");
                    };
                }
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
                self.event_pub.send(Event::EndpointAdded { epid, total }).await?;
            }
            Event::EndpointRemoved { epid, total } => {
                self.event_pub.send(Event::EndpointRemoved { epid, total }).await?;
            }
            Event::NewConnection { ep, origin, sender } => {
                let epid = ep.id;
                let addr = ep.address;

                outbox.insert(epid, sender);
                connected.insert(ep);

                self.event_pub
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
                    self.event_pub
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
                    None,
                    &mut self.event_pub_intern,
                )
                .await?;
            }
            Event::MessageSent { epid, num_bytes } => {
                self.event_pub.send(Event::MessageSent { epid, num_bytes }).await?
            }
            Event::MessageReceived { epid, bytes } => {
                self.event_pub.send(Event::MessageReceived { epid, bytes }).await?
            }
            Event::TryConnect { epid, responder } => {
                try_connect(
                    epid,
                    self.reconnect_interval,
                    contacts,
                    connected,
                    responder,
                    &mut self.event_pub_intern,
                )
                .await?;
            }
            _ => (),
        }

        Ok(true)
    }
}

#[inline(always)]
async fn add_endpoint(contacts: &mut Endpoints, url: Url, notifier: &mut EventPublisher) -> Result<bool, WorkerError> {
    let ep = Ep::from_url(url);
    let epid = ep.id;

    if contacts.insert(ep) {
        // add its ip to the allowlist, so that we can make sure that we accept only connections
        // from known peers
        let allowlist = allowlist::get();
        allowlist.insert(epid, url.address().ip());

        notifier
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

#[inline(always)]
async fn remove_endpoint(
    epid: EpId,
    contacts: &mut Endpoints,
    connected: &mut Endpoints,
    outbox: &mut Outbox,
    notifier: &mut EventPublisher,
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

        notifier
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

#[inline(always)]
async fn try_connect(
    epid: EpId,
    reconnect_interval: Duration,
    contacts: &mut Endpoints,
    connected: &mut Endpoints,
    responder: Option<Responder<bool>>,
    notifier: &mut EventPublisher,
) -> Result<bool, WorkerError> {
    // Try to find the endpoint in our servers list.
    if let Some(ep) = contacts.get_mut(&epid) {
        // if ep.is_connected() {
        if connected.contains(&ep.id) {
            if let Some(responder) = responder {
                match responder.send(false) {
                    Ok(_) => (),
                    Err(_) => {
                        error!("Failed to send command response.");
                    }
                }
            }
            Ok(false)
        } else {
            match ep.protocol {
                Protocol::Tcp => {
                    if tcp::try_connect(&ep.id, &ep.address, notifier.clone()).await.is_ok() {
                        connected.insert(ep.clone());
                        if let Some(responder) = responder {
                            match responder.send(true) {
                                Ok(_) => (),
                                Err(_) => {
                                    error!("Failed to send command response.");
                                }
                            }
                        }
                        Ok(true)
                    } else {
                        // If connection attempt fails, issue a `TryConnect` event after a certain delay.
                        // NOTE: It won't be raised, if the endpoint has been removed in the mean time.
                        spawn(raise_event_after_delay(
                            Event::TryConnect { epid, responder },
                            reconnect_interval,
                            notifier.clone(),
                        ));
                        Ok(false)
                    }
                }
                Protocol::Udp => {
                    if let Some(responder) = responder {
                        match responder.send(true) {
                            Ok(_) => (),
                            Err(_) => {
                                error!("Failed to send response.");
                            }
                        }
                    }
                    Ok(true)
                }
            }
        }
    } else {
        if let Some(responder) = responder {
            match responder.send(false) {
                Ok(_) => (),
                Err(_) => {
                    error!("Failed to send response.");
                }
            }
        }
        Ok(false)
    }
}

#[inline(always)]
async fn raise_event_after_delay(
    event: Event,
    delay: Duration,
    mut notifier: EventPublisher,
) -> Result<(), WorkerError> {
    task::sleep(delay).await;

    Ok(notifier.send(event).await?)
}

#[inline(always)]
async fn disconnect(epid: EpId, connected: &mut Endpoints, outbox: &mut Outbox) -> bool {
    let removed_recipient = outbox.remove(&epid);
    let removed_connected = connected.remove(&epid);

    if removed_connected && !removed_recipient {
        warn!("Removed an endpoint that was connected, but couldn't be sent to.");
    }

    removed_connected
}

#[inline(always)]
async fn send_bytes(recipient: &EpId, bytes: Vec<u8>, outbox: &mut Outbox) -> Result<bool, WorkerError> {
    Ok(outbox.send(bytes, recipient).await?)
}

#[inline(always)]
async fn multicast_bytes(recipients: &[EpId], bytes: Vec<u8>, outbox: &mut Outbox) -> Result<bool, WorkerError> {
    Ok(outbox.multicast(bytes, recipients).await?)
}

#[inline(always)]
async fn broadcast_bytes(bytes: Vec<u8>, outbox: &mut Outbox) -> Result<bool, WorkerError> {
    Ok(outbox.broadcast(bytes).await?)
}

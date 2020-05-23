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

use super::whitelist;

use crate::{
    address::url::{Protocol, Url},
    commands::{Command, CommandReceiver as Commands, Responder},
    constants::CONNECT_INTERVAL,
    endpoint::{outbox::Outbox, store::Endpoints, Endpoint as Ep, EndpointId as EpId},
    errors::Result,
    events::{Event, EventPublisher as Notifier, EventPublisher as Publisher, EventSubscriber as Events},
    shutdown::ShutdownListener as Shutdown,
    tcp,
    utils::time,
};

use async_std::{
    prelude::*,
    task::{self, spawn},
};
use futures::{select, sink::SinkExt, FutureExt};
use log::*;

use std::time::Duration;

pub struct EndpointWorker {
    commands: Commands,
    events: Events,
    shutdown: Shutdown,
    notifier: Notifier,
    publisher: Publisher,
}

impl EndpointWorker {
    pub fn new(
        commands: Commands,
        events: Events,
        shutdown: Shutdown,
        notifier: Notifier,
        publisher: Publisher,
    ) -> Self {
        Self {
            commands,
            events,
            shutdown,
            notifier,
            publisher,
        }
    }

    pub async fn run(mut self) -> Result<()> {
        debug!("Starting endpoint worker...");

        let mut contacts = Endpoints::new();

        // TODO: those two probably need to be merged as each connected endpoint is also part of the outbox
        let mut connected = Endpoints::new();
        let mut outbox = Outbox::new();

        let commands = &mut self.commands;
        let events = &mut self.events;
        let shutdown = &mut self.shutdown;
        let publisher = &mut self.publisher;

        loop {
            select! {
                command = commands.next().fuse() => {

                    let command = if let Some(command) = command {
                        command
                    } else {
                        error!("Command channel unexpectedly closed.");
                        break;
                    };

                    debug!("Received {}.", command);

                    match command {
                        Command::AddEndpoint { url, responder } => {
                            let res = add_endpoint(&mut contacts, url, &mut self.notifier).await?;

                            if let Some(responder) = responder {
                                if responder.send(res).is_err() {
                                    warn!("Error sending command response.");
                                };
                            }
                        },
                        Command::RemoveEndpoint { epid, responder } => {
                            let res = rmv_endpoint(epid, &mut contacts, &mut connected, &mut outbox,
                                &mut self.notifier).await?;

                            if let Some(responder) = responder {
                                if responder.send(res).is_err() {
                                    warn!("Error sending command response.");
                                };
                            }
                        },
                        Command::Connect { epid, responder } => {
                            try_connect(epid, &mut contacts, &mut connected, responder, &mut self.notifier).await?;
                        },
                        Command::Disconnect { epid, responder } => {
                            let is_disconnected = disconnect(epid, &mut connected, &mut outbox).await;

                            if let Some(responder) = responder {
                                if responder.send(is_disconnected).is_err() {
                                    warn!("Error sending command response.");
                                };
                            }

                            if is_disconnected {
                                publisher
                                    .send(Event::EndpointDisconnected {
                                        epid,
                                        total: connected.num(),
                                    })
                                    .await?;
                            }

                        },
                        Command::SendMessage { epid, bytes, responder } => {
                            let res = send_bytes(&epid, bytes, &mut outbox).await?;

                            if let Some(responder) = responder {
                                if responder.send(res).is_err() {
                                    warn!("Error sending command response.");
                                };
                            }
                        },
                        Command::MulticastMessage { epids, bytes, responder } => {
                            let res = multicast_bytes(&epids, bytes, &mut outbox).await?;

                            if let Some(responder) = responder {
                                if responder.send(res).is_err() {
                                    warn!("Error sending command response.");
                                };
                            }
                        },
                        Command::BroadcastMessage { bytes, responder } => {
                            let res = broadcast_bytes(bytes, &mut outbox).await?;

                            if let Some(responder) = responder {
                                if responder.send(res).is_err() {
                                    warn!("Error sending command response.");
                                };
                            }
                        },
                    }

                },
                event = events.next().fuse() => {
                    let event = if let Some(event) = event {
                        event
                    } else {
                        error!("Event channel unexpectedly closed.");
                        break;
                    };

                    debug!("Received {}.", event);

                    match event {
                        Event::EndpointAdded { epid, total } => {
                            publisher.send(Event::EndpointAdded { epid, total }).await?;
                        },
                        Event::EndpointRemoved { epid, total } => {
                            publisher.send(Event::EndpointRemoved { epid, total }).await?;
                        },
                        Event::NewConnection { ep, origin, sender } => {
                            let epid = ep.id;
                            let addr = ep.address;

                            outbox.insert(epid, sender);
                            connected.insert(ep);

                            publisher.send(Event::EndpointConnected {
                                epid,
                                address: addr,
                                origin,
                                timestamp: time::timestamp_millis(),
                                total: connected.num(),
                            }).await?
                        },
                        Event::LostConnection { epid } => {
                            let is_disconnected = disconnect(epid, &mut connected, &mut outbox).await;

                            if is_disconnected {
                                publisher
                                    .send(Event::EndpointDisconnected {
                                        epid,
                                        total: connected.num(),
                                    })
                                    .await?;
                            }

                            // TODO: do not try to reconnect to duplicate endpoints
                            // NOTE: 'try_connect' will check if 'epid' is part of the contact list
                            try_connect(epid, &mut contacts, &mut connected, None, &mut self.notifier).await?;
                        }
                        Event::MessageSent { epid, num_bytes } => {
                            publisher.send(Event::MessageSent {
                                epid,
                                num_bytes,
                            }).await?
                        },
                        Event::MessageReceived { epid, bytes } => {
                            publisher.send(Event::MessageReceived {
                                epid,
                                bytes,
                            }).await?
                        },
                        Event::TryConnect { epid, responder } => {
                            try_connect(epid, &mut contacts, &mut connected, responder, &mut self.notifier).await?;
                        }
                        _ => (),
                    }
                },

                shutdown = shutdown.fuse() => {
                    break;
                }
            }
        }

        debug!("Stopped endpoint worker.");
        Ok(())
    }
}

#[inline(always)]
async fn add_endpoint(contacts: &mut Endpoints, url: Url, notifier: &mut Notifier) -> Result<bool> {
    let ep = Ep::from_url(url);
    let epid = ep.id;

    if contacts.insert(ep) {
        // add its ip to the whitelist, so that we can make sure that we accept only connections
        // from known peers
        let whitelist = whitelist::get();
        whitelist.insert(epid, url.address().ip());

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
async fn rmv_endpoint(
    epid: EpId,
    contacts: &mut Endpoints,
    connected: &mut Endpoints,
    outbox: &mut Outbox,
    notifier: &mut Notifier,
) -> Result<bool> {
    // NOTE: current default behavior is to drop connections once the contact is removed
    let removed_recipient = outbox.remove(&epid);
    let removed_contact = contacts.remove(&epid);
    let removed_connected = connected.remove(&epid);

    if removed_connected && !removed_recipient {
        warn!("Removed an endpoint that was connected, but couldn't be sent to.");
    }

    if removed_contact || removed_connected {
        // Remove its IP also from the whitelist, so we won't accept connections from it
        // anymore
        let whitelist = whitelist::get();
        whitelist.remove(&epid);

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
    contacts: &mut Endpoints,
    connected: &mut Endpoints,
    responder: Option<Responder<bool>>,
    notifier: &mut Notifier,
) -> Result<bool> {
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
                            CONNECT_INTERVAL,
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
async fn raise_event_after_delay(event: Event, delay: u64, mut notifier: Notifier) -> Result<()> {
    task::sleep(Duration::from_millis(delay)).await;

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
async fn send_bytes(recipient: &EpId, bytes: Vec<u8>, outbox: &mut Outbox) -> Result<bool> {
    Ok(outbox.send(bytes, recipient).await?)
}

#[inline(always)]
async fn multicast_bytes(recipients: &[EpId], bytes: Vec<u8>, outbox: &mut Outbox) -> Result<bool> {
    Ok(outbox.multicast(bytes, recipients).await?)
}

#[inline(always)]
async fn broadcast_bytes(bytes: Vec<u8>, outbox: &mut Outbox) -> Result<bool> {
    Ok(outbox.broadcast(bytes).await?)
}

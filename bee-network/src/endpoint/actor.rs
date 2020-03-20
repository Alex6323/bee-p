use crate::address::url::{
    Protocol,
    Url,
};

use crate::commands::CommandReceiver as Commands;
use crate::commands::{
    Command,
    Responder,
};
use crate::constants::CONNECT_INTERVAL;
use crate::endpoint::{
    outbox::Outbox,
    store::Endpoints,
    Endpoint as Ep,
    EndpointId as EpId,
};
use crate::errors::{
    ActorResult as R,
    ActorSuccess as S,
};
use crate::events::Event;
use crate::events::EventPublisher as Notifier;
use crate::events::EventPublisher as Publisher;
use crate::events::EventSubscriber as Events;
use crate::shutdown::ShutdownListener as Shutdown;
use crate::tcp;
use crate::utils::time;

use async_std::prelude::*;
use async_std::task::{
    self,
    spawn,
};
use futures::sink::SinkExt;
use futures::{
    select,
    FutureExt,
};
use log::*;

use std::time::Duration;

pub struct EndpointActor {
    commands: Commands,
    events: Events,
    shutdown: Shutdown,
    notifier: Notifier,
    publisher: Publisher,
}

impl EndpointActor {
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

    pub async fn run(mut self) -> S {
        debug!("[Endp ] Starting actor");

        let mut contacts = Endpoints::new();

        // TODO: those two probably need to be merged as each connected endpoint is also part of the outbox
        let mut connected = Endpoints::new();
        let mut outbox = Outbox::new();

        let commands = &mut self.commands;
        let events = &mut self.events;
        let shutdown = &mut self.shutdown;

        loop {
            select! {
                command = commands.next().fuse() => {

                    let command = if let Some(command) = command {
                        command
                    } else {
                        error!("[Endp ] Command channel unexpectedly closed");
                        break;
                    };

                    debug!("[Endp ] Received {} command.", command);

                    match command {
                        Command::AddEndpoint { url, responder } => {
                            let res = add_endpoint(&mut contacts, url, &mut self.notifier).await?;

                            if let Some(responder) = responder {
                                responder.send(res);
                            }
                        },
                        Command::RemoveEndpoint { epid, responder } => {
                            let res = rem_endpoint(epid, &mut contacts, &mut connected, &mut outbox,
                                &mut self.notifier).await?;

                            if let Some(responder) = responder {
                                responder.send(res);
                            }
                        },
                        Command::Connect { epid, responder } => {
                            try_connect(epid, &mut contacts, &mut connected, responder, &mut self.notifier).await?;
                        },
                        Command::Disconnect { epid, responder } => {
                            let res = disconnect(epid, &mut connected, &mut outbox, &mut self.publisher).await?;

                            if let Some(responder) = responder {
                                responder.send(res);
                            }
                        },
                        Command::SendBytes { epid, bytes, responder } => {
                            let res = send_bytes(&epid, bytes, &mut outbox).await?;

                            if let Some(responder) = responder {
                                responder.send(res);
                            }
                        },
                        Command::MulticastBytes { epids, bytes, responder } => {
                            let res = multicast_bytes(&epids, bytes, &mut outbox).await?;

                            if let Some(responder) = responder {
                                responder.send(res);
                            }
                        },
                        Command::BroadcastBytes { bytes, responder } => {
                            let res = broadcast_bytes(bytes, &mut outbox).await?;

                            if let Some(responder) = responder {
                                responder.send(res);
                            }
                        },
                    }

                },
                event = events.next().fuse() => {
                    let event = if let Some(event) = event {
                        event
                    } else {
                        error!("[Endp ] Event channel unexpectedly closed");
                        break;
                    };

                    debug!("[Endp ] Received {} event.", event);

                    match event {
                        Event::EndpointAdded { epid, total } => {
                            self.publisher.send(Event::EndpointAdded { epid, total }).await;
                        },
                        Event::EndpointRemoved { epid, total } => {
                            self.publisher.send(Event::EndpointRemoved { epid, total }).await;
                        },
                        Event::NewConnection { ep, sender } => {
                            let epid = ep.id;

                            outbox.insert(epid, sender);
                            connected.insert(ep);

                            self.publisher.send(Event::EndpointConnected {
                                epid,
                                timestamp: time::timestamp_millis(),
                                total: connected.num(),
                            }).await?
                        },
                        Event::LostConnection { epid } => {
                            disconnect(epid, &mut connected, &mut outbox, &mut self.publisher).await?;

                            // NOTE: 'try_connect' will check if 'epid' is part of the contact list
                            try_connect(epid, &mut contacts, &mut connected, None, &mut self.notifier).await?;
                        }
                        Event::BytesSent { epid, num } => {
                            self.publisher.send(Event::BytesSent {
                                epid,
                                num,
                            }).await?
                        },
                        Event::BytesReceived { epid, addr, bytes } => {
                            self.publisher.send(Event::BytesReceived {
                                epid,
                                addr,
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

        debug!("[Endp ] Stopping actor");
        Ok(())
    }
}

#[inline(always)]
async fn add_endpoint(contacts: &mut Endpoints, url: Url, notifier: &mut Notifier) -> R<bool> {
    let ep = Ep::from_url(url);
    let epid = ep.id;

    if contacts.insert(ep) {
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
async fn rem_endpoint(
    epid: EpId,
    contacts: &mut Endpoints,
    connected: &mut Endpoints,
    outbox: &mut Outbox,
    notifier: &mut Notifier,
) -> R<bool> {
    // NOTE: current default behavior is to drop connections once the contact is removed
    let removed_recipient = outbox.remove(&epid);
    let removed_contact = contacts.remove(&epid);
    let removed_connected = connected.remove(&epid);

    if removed_connected && !removed_recipient {
        warn!("[Endp ] Removed an endpoint that was connected, but couldn't be sent to.");
    }

    if removed_contact || removed_connected {
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
) -> R<bool> {
    // Try to find the endpoint in our servers list.
    if let Some(ep) = contacts.get_mut(&epid) {
        //if ep.is_connected() {
        if connected.contains(&ep.id) {
            if let Some(responder) = responder {
                match responder.send(false) {
                    Ok(_) => (),
                    Err(e) => {
                        error!("[Endp ] Failed to send response: {}", e);
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
                                Err(e) => {
                                    error!("[Endp ] Failed to send response: {}", e);
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
                            Err(e) => {
                                error!("[Endp ] Failed to send response: {}", e);
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
                Err(e) => {
                    error!("[Endp ] Failed to send response: {}", e);
                }
            }
        }
        Ok(false)
    }
}

#[inline(always)]
async fn raise_event_after_delay(event: Event, delay: u64, mut notifier: Notifier) -> S {
    task::sleep(Duration::from_millis(delay)).await;

    Ok(notifier.send(event).await?)
}

#[inline(always)]
async fn disconnect(epid: EpId, connected: &mut Endpoints, outbox: &mut Outbox, publisher: &mut Publisher) -> R<bool> {
    let removed_recipient = outbox.remove(&epid);
    let removed_connected = connected.remove(&epid);

    if removed_connected && !removed_recipient {
        warn!("[Endp ] Removed an endpoint that was connected, but couldn't be sent to.");
    }

    if removed_connected {
        publisher
            .send(Event::EndpointDisconnected {
                epid,
                total: connected.num(),
            })
            .await?;
        Ok(true)
    } else {
        Ok(false)
    }
}

#[inline(always)]
async fn send_bytes(recipient: &EpId, bytes: Vec<u8>, outbox: &mut Outbox) -> R<bool> {
    Ok(outbox.send(bytes, recipient).await?)
}

#[inline(always)]
async fn multicast_bytes(recipients: &Vec<EpId>, bytes: Vec<u8>, outbox: &mut Outbox) -> R<bool> {
    Ok(outbox.multicast(bytes, recipients).await?)
}

#[inline(always)]
async fn broadcast_bytes(bytes: Vec<u8>, outbox: &mut Outbox) -> R<bool> {
    Ok(outbox.broadcast(bytes).await?)
}

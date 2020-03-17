use crate::address::url::{
    Protocol,
    Url,
};

use crate::commands::CommandReceiver as Commands;
use crate::commands::{
    Command,
    Responder,
};
use crate::connection::ConnectionPool;
use crate::endpoint::pool::EndpointPool;
use crate::endpoint::{
    Endpoint,
    EndpointId,
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

use async_std::prelude::*;
use async_std::sync::Arc;
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

const CONNECT_INTERVAL: u64 = 5000;

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

        let mut servers = EndpointPool::new();
        let mut clients = EndpointPool::new();

        let mut tconns = ConnectionPool::new();
        let mut uconns = ConnectionPool::new();

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

                    match command {
                        Command::AddEndpoint { url, responder } => {
                            let res = add_endpoint(&mut servers, &clients, url, &mut self.notifier).await?;

                            if let Some(responder) = responder {
                                responder.send(res);
                            }
                        },
                        Command::RemoveEndpoint { id, responder } => {
                            let res = rem_endpoint(&mut servers, &mut clients, &mut tconns, &mut uconns, id,
                                &mut self.notifier).await?;

                            if let Some(responder) = responder {
                                responder.send(res);
                            }
                        },
                        Command::Connect { to, responder } => {
                            try_connect(&mut servers, to, responder, &mut self.notifier).await?;
                        },
                        Command::Disconnect { from, responder } => {
                            let res = disconnect(&mut servers, &mut clients, &mut tconns, &mut uconns, from, &mut self.notifier).await?;

                            if let Some(responder) = responder {
                                responder.send(res);
                            }
                        },
                        Command::SendBytes { to, bytes, responder } => {
                            let res = send_bytes(&mut tconns, &mut uconns, bytes, &to).await?;

                            if let Some(responder) = responder {
                                responder.send(res);
                            }
                        },
                        Command::MulticastBytes { to, bytes, responder } => {
                            let res = multicast_bytes(&mut tconns, &mut uconns, bytes, &to).await?;

                            if let Some(responder) = responder {
                                responder.send(res);
                            }
                        },
                        Command::BroadcastBytes { bytes, responder } => {
                            let res = broadcast_bytes(&mut tconns, &mut uconns, bytes).await?;

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

                    match event {
                        Event::EndpointAdded { epid, total } => {
                            self.publisher.send(Event::EndpointAdded { epid, total }).await;
                        },
                        Event::EndpointRemoved { epid, total } => {
                            self.publisher.send(Event::EndpointRemoved { epid, total }).await;
                        },
                        Event::NewConnection { epid, addr, prot, sender } => {
                            // TODO
                        },
                        Event::EndpointConnected { epid, timestamp, total } => {
                            // TODO
                        },
                        Event::EndpointDisconnected { epid, total } => {
                            // TODO
                        },
                        Event::BytesSent { to, num } => {
                            // TODO
                        },
                        Event::BytesReceived { from, with_addr, bytes } => {
                            // TODO
                        },
                        Event::TryConnect { to, responder } => {
                            try_connect(&mut servers, to, responder, &mut self.notifier).await?;
                        }
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
async fn add_endpoint(
    servers: &mut EndpointPool,
    clients: &EndpointPool,
    url: Url,
    notifier: &mut Notifier,
) -> R<bool> {
    let ep = Endpoint::from_url(url.clone());
    let epid = ep.id.clone();

    let added_server = servers.insert(ep);

    if added_server {
        notifier
            .send(Event::EndpointAdded {
                epid,
                total: servers.size() + clients.size(),
            })
            .await?;
        Ok(true)
    } else {
        Ok(false)
    }
}

#[inline(always)]
async fn rem_endpoint(
    servers: &mut EndpointPool,
    clients: &mut EndpointPool,
    tconns: &mut ConnectionPool,
    uconns: &mut ConnectionPool,
    epid: EndpointId,
    notifier: &mut Notifier,
) -> R<bool> {
    // NOTE: current default behavior is to drop connections as well
    tconns.remove(&epid);
    uconns.remove(&epid);

    let removed_server = servers.remove(&epid);
    let removed_client = clients.remove(&epid);

    if removed_server || removed_client {
        notifier
            .send(Event::EndpointRemoved {
                epid,
                total: clients.size() + servers.size(),
            })
            .await?;
        Ok(true)
    } else {
        Ok(false)
    }
}

#[inline(always)]
async fn try_connect(
    servers: &mut EndpointPool,
    epid: EndpointId,
    responder: Option<Responder<bool>>,
    notifier: &mut Notifier,
) -> R<bool> {
    // Try to find the endpoint in our servers list.
    if let Some(ep) = servers.get_mut(&epid) {
        // If already connected, do nothing.
        if ep.is_connected() {
            if let Some(responder) = responder {
                responder.send(false);
            }
            Ok(false)
        } else {
            match ep.protocol {
                Protocol::Tcp => {
                    // If the connection attempt succeeds, change the endpoint's state.
                    if tcp::try_connect(&ep.id, &ep.address, notifier.clone()).await.is_ok() {
                        ep.set_connected();
                        if let Some(responder) = responder {
                            responder.send(true);
                        }
                        Ok(true)
                    } else {
                        // If connection attempt fails, issue a `TryConnect` event after a certain delay.
                        // NOTE: It won't be raised, if the endpoint has been removed in the mean time.
                        spawn(raise_event_after_delay(
                            Event::TryConnect { to: epid, responder },
                            CONNECT_INTERVAL,
                            notifier.clone(),
                        ));
                        Ok(false)
                    }
                }
                Protocol::Udp => {
                    if let Some(responder) = responder {
                        responder.send(true);
                    }
                    Ok(true)
                }
            }
        }
    } else {
        if let Some(responder) = responder {
            responder.send(false);
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
async fn disconnect(
    servers: &mut EndpointPool,
    clients: &mut EndpointPool,
    tconns: &mut ConnectionPool,
    uconns: &mut ConnectionPool,
    epid: EndpointId,
    notifier: &mut Notifier,
) -> R<bool> {
    // NOTE: By removing the `BytesSender`s we are exiting their event loops,
    // which completes their futures/finishes their tasks handling the connection.
    let removed_tconn = tconns.remove(&epid);
    let removed_uconn = uconns.remove(&epid);

    if removed_tconn || removed_uconn {
        if let Some(server_ep) = servers.get_mut(&epid) {
            server_ep.set_disconnected();
        } else if let Some(client_ep) = clients.get_mut(&epid) {
            client_ep.set_disconnected();
        }

        notifier
            .send(Event::EndpointDisconnected {
                epid,
                total: tconns.size() + uconns.size(),
            })
            .await?;
        Ok(true)
    } else {
        Ok(false)
    }
}

#[inline(always)]
async fn send_bytes(
    tconns: &mut ConnectionPool,
    uconns: &mut ConnectionPool,
    bytes: Vec<u8>,
    receiver: &EndpointId,
) -> R<bool> {
    let bytes = Arc::new(bytes);

    // FIXME: Make it so that we can spawn two tasks (so that sending can happen in parallel)

    let sent_over_tcp = tconns.send(Arc::clone(&bytes), receiver).await?;
    let sent_over_udp = uconns.send(bytes, receiver).await?;

    Ok(sent_over_tcp || sent_over_udp)
}

#[inline(always)]
async fn multicast_bytes(
    tconns: &mut ConnectionPool,
    uconns: &mut ConnectionPool,
    bytes: Vec<u8>,
    receivers: &Vec<EndpointId>,
) -> R<bool> {
    let bytes = Arc::new(bytes);

    // FIXME: Make it so that we can spawn two tasks (so that sending can happen in parallel)

    let sent_over_tcp = tconns.multicast(Arc::clone(&bytes), receivers).await?;
    let sent_over_udp = uconns.multicast(bytes, receivers).await?;

    Ok(sent_over_tcp || sent_over_udp)
}

#[inline(always)]
async fn broadcast_bytes(tconns: &mut ConnectionPool, uconns: &mut ConnectionPool, bytes: Vec<u8>) -> R<bool> {
    let bytes = Arc::new(bytes);

    // FIXME: Make it so that we can spawn two tasks (so that sending can happen in parallel)

    let sent_over_tcp = tconns.broadcast(Arc::clone(&bytes)).await?;
    let sent_over_udp = uconns.broadcast(bytes).await?;

    Ok(sent_over_tcp || sent_over_udp)
}

use crate::address::url::{
    Protocol,
    Url,
};
use crate::commands::CommandReceiver as Commands;
use crate::commands::{
    Command,
    Responder,
};
use crate::connection::{
    BytesReceiver,
    BytesSender,
    ConnectionPool,
};
use crate::endpoint::pool::EndpointPool;
use crate::endpoint::{
    Endpoint,
    EndpointId,
};
use crate::events::Event;
use crate::events::EventPublisher as Notifier;
use crate::events::EventPublisher as Publisher;
use crate::events::EventSubscriber as Events;
use crate::shutdown::ShutdownListener as Shutdown;
use crate::tcp;
use crate::{
    R,
    R0,
};

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

use std::collections::HashMap;
use std::time::Duration;

type AddEpResponder = Option<Responder<Option<EndpointId>>>;
type RemEpResponder = Option<Responder<bool>>;
type ConnectionResponder = Option<Responder<bool>>;

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

    pub async fn run(mut self) -> R0 {
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
                            add_endpoint(&mut servers, &clients, url, responder, &mut self.notifier).await?;
                        },
                        Command::RemoveEndpoint { id, responder } => {
                            rem_endpoint(&mut servers, &mut clients, &mut tconns, &mut uconns, id, responder,
                                &mut self.notifier).await?;
                        },
                        Command::Connect { to, responder } => {
                            try_connect(&mut servers, to, responder, &mut self.notifier).await?;
                        },
                        Command::Disconnect { from, responder } => {
                            // TODO
                        },
                        Command::SendBytes { to, bytes } => {
                            // TODO
                        },
                        Command::MulticastBytes { to, bytes } => {
                            // TODO
                        },
                        Command::BroadcastBytes { bytes } => {
                            // TODO
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
                        Event::EndpointAdded { id, total } => {
                            // TODO
                        },
                        Event::EndpointRemoved { id, total } => {
                            // TODO
                        },
                        Event::EndpointAccepted { id, url, sender } => {
                            // TODO
                        },
                        Event::ConnectionEstablished { id, timestamp, total } => {
                            // TODO
                        },
                        Event::ConnectionDropped { id, total } => {
                            // TODO
                        },
                        Event::BytesSent { to, num } => {
                            // TODO
                        },
                        Event::BytesReceived { from, with_addr, num, buffer } => {
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
    responder: AddEpResponder,
    notifier: &mut Notifier,
) -> R0 {
    let ep = Endpoint::from_url(url.clone());
    let id = ep.id.clone();

    servers.insert(ep);

    if let Some(responder) = responder {
        responder.send(Some(id.clone()));
    }

    Ok(notifier
        .send(Event::EndpointAdded {
            id,
            total: servers.size() + clients.size(),
        })
        .await?)
}

#[inline(always)]
async fn rem_endpoint(
    servers: &mut EndpointPool,
    clients: &mut EndpointPool,
    tconns: &mut ConnectionPool,
    uconns: &mut ConnectionPool,
    id: EndpointId,
    responder: RemEpResponder,
    notifier: &mut Notifier,
) -> R0 {
    let removed_server = servers.remove(&id);
    let removed_client = clients.remove(&id);

    // NOTE: current default behavior is to drop connections as well
    let removed_tconn = tconns.remove(&id);
    let removed_uconn = uconns.remove(&id);

    if let Some(responder) = responder {
        responder.send(removed_server || removed_client);
    }

    Ok(notifier
        .send(Event::EndpointRemoved {
            id,
            total: clients.size() + servers.size(),
        })
        .await?)
}

#[inline(always)]
async fn try_connect(
    servers: &mut EndpointPool,
    to: EndpointId,
    responder: ConnectionResponder,
    notifier: &mut Notifier,
) -> R<bool> {
    // Try to find the endpoint in our servers list.
    if let Some(ep) = servers.get_mut(&to) {
        // If already connected, do nothing.
        if ep.is_connected() {
            Ok(false)
        } else {
            match ep.protocol {
                Protocol::Tcp => {
                    // If the connection attempt succeeds, change the endpoint's state.
                    if tcp::connect(&ep.id, &ep.address, notifier.clone()).await {
                        ep.set_connected();
                        Ok(true)
                    } else {
                        // If connection attempt fails, issue a `TryConnect` event after a certain delay.
                        // NOTE: It won't be raised, if the endpoint has been removed in the mean time.
                        spawn(raise_event_after_delay(
                            Event::TryConnect { to, responder },
                            CONNECT_INTERVAL,
                            notifier.clone(),
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

#[inline(always)]
async fn raise_event_after_delay(event: Event, delay: u64, mut notifier: Notifier) -> R0 {
    task::sleep(Duration::from_millis(delay)).await;

    Ok(notifier.send(event).await?)
}

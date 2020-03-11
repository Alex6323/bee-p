use crate::commands::Command;
use crate::commands::CommandReceiver as Commands;
use crate::connection::{
    BytesReceiver,
    BytesSender,
    ConnectionPool,
};
use crate::endpoint::pool::EndpointPool;
use crate::endpoint::EndpointId;
use crate::events::Event;
use crate::events::EventPublisher as EventPub;
use crate::events::EventSubscriber as Events;
use crate::shutdown::ShutdownListener;

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

pub struct EndpointActor {
    commands: Commands,
    internals: Events,
    notifier: EventPub,
    publisher: EventPub,
    shutdown: ShutdownListener,
}

impl EndpointActor {
    pub fn new(
        commands: Commands,
        internals: Events,
        notifier: EventPub,
        publisher: EventPub,
        shutdown: ShutdownListener,
    ) -> Self {
        Self {
            commands,
            internals,
            notifier,
            publisher,
            shutdown,
        }
    }

    pub async fn run(mut self) {
        debug!("[Edp  ] Starting actor");

        let mut server_endpoints = EndpointPool::new();
        let mut client_endpoints = EndpointPool::new();

        let mut tcp_conns = ConnectionPool::new();
        let mut udp_conns = ConnectionPool::new();

        let commands = &mut self.commands;
        let internals = &mut self.internals;

        loop {
            select! {
                command = commands.next().fuse() => {
                    break;
                }
                internal = internals.next().fuse() => {
                    break;
                }
            }
        }
        /*
        loop {

            select! {
                // === Handle commands ===
                command = command_rx.next().fuse() => {
                    if command.is_none() {
                        debug!("[Peers] Commands channel closed");
                        break;
                    }
                    let command = command.unwrap();
                    info!("[Peers] {:?}", command);

                    match command {
                        Command::AddPeer { mut peer, connect_attempts } => {
                            server_peers.add(peer.clone());

                            event_pub.send(Event::PeerAdded {
                                peer_id: peer.id(),
                                num_peers: server_peers.num() }).await;

                            let retry = match connect_attempts {
                                Some(0) => Some(0),
                                Some(n) => Some(n-1),
                                None => continue,
                            };

                            event_pub.send(Event::TryConnect { peer_id: peer.id(), retry }).await;
                        },
                        Command::RemovePeer { peer_id } => {
                            server_peers.remove(&peer_id);

                            // TODO: check if this correctly ends the connection actor
                            tcp_conns.remove(&peer_id);
                            udp_conns.remove(&peer_id);

                            event_pub.send(Event::PeerRemoved {
                                peer_id,
                                num_peers: server_peers.num() }).await;
                        },
                        Command::Connect { peer_id, num_retries } => {
                            if !server_peers.contains(&peer_id) {
                                continue;
                            }

                            event_pub.send(Event::TryConnect { peer_id, retry: Some(num_retries) }).await;
                        },
                        Command::SendBytes { to_peer, bytes } => {
                            let num_bytes = bytes.len();

                            if let Some(sender) = tcp_conns.get_mut(&to_peer) {
                                sender.send(bytes).await;

                            } else if let Some(sender) = udp_conns.get_mut(&to_peer) {
                                sender.send(bytes).await;

                            } else {
                                warn!("[Peers] No connection with peer {:?}", to_peer);
                                continue;
                            }

                            //event_pub.send(Event::BytesSent { to_peer, num_bytes }).await;
                        },
                        Command::BroadcastBytes { bytes } => {
                            let mut num_conns = 0;

                            // TODO: send concurrently
                            // TODO: do not clone the bytes (use Arc!)
                            for (_, sender) in tcp_conns.iter_mut() {
                                sender.send(bytes.clone()).await;
                                num_conns += 1;
                            }

                            // TODO: send concurrently
                            // TODO: do not clone the bytes (use Arc!)
                            for (_, sender) in udp_conns.iter_mut() {
                                sender.send(bytes.clone()).await;
                                num_conns += 1;
                            }

                            if num_conns == 0 {
                                warn!("[Peers] No connections available for broadcast.");
                                continue;
                            }

                            // FIXME: this never shows in the logs. Why?
                            event_pub.send(Event::BytesBroadcasted {
                                num_bytes: bytes.len(),
                                num_conns,
                            });
                        },
                        Command::Shutdown => {
                            drop(tcp_conns);
                            drop(udp_conns);
                            drop(server_peers);
                            drop(client_peers);
                            break;
                        }
                    }
                },

                // === Handle peer events ===
                peer_event = event_sub.next().fuse() => {

                    // TODO: replace this with 'unwrap_or_else(|| break)'
                    if peer_event.is_none() {
                        debug!("[Peers] Event channel closed");
                        break;
                    }
                    let peer_event = peer_event.unwrap();
                        info!("[Peers] {:?}", peer_event);

                    match peer_event {
                        Event::PeerAdded { peer_id, num_peers } => {
                            // notify user
                            net_pub.send(Event::PeerAdded { peer_id, num_peers }).await;
                        },
                        Event::PeerRemoved { peer_id, num_peers } => {
                            // notify user
                            net_pub.send(Event::PeerRemoved { peer_id, num_peers }).await;
                        },
                        Event::PeerAccepted { peer_id, peer_url, sender } => {
                            use Protocol::*;

                            let protocol = peer_url.protocol();

                            // NOTE: we let users deal with duplicate peers because for that
                            // handshakes are required.
                            if !server_peers.contains(&peer_id) &&
                               !client_peers.contains(&peer_id) {
                                client_peers.add(Peer::from_url(peer_url));
                            }

                            // TODO: use Entry API
                            match protocol {
                                Tcp => {
                                    if !tcp_conns.contains_key(&peer_id) {
                                        tcp_conns.insert(peer_id, sender);
                                    }
                                },
                                Udp => {
                                    if !udp_conns.contains_key(&peer_id) {
                                        udp_conns.insert(peer_id, sender);
                                    }
                                },
                                _ => (),
                            }

                            let num_conns = tcp_conns.len() + udp_conns.len();
                            let timestamp = utils::timestamp_millis();

                            event_pub.send(Event::PeerConnected { peer_id, num_conns, timestamp }).await
                                .expect("[Peers] Error sending 'PeerConnected' event");
                        }
                        Event::PeerConnected { peer_id, num_conns, timestamp } => {

                            let mut peer = {
                                if let Some(peer) = server_peers.get_mut(&peer_id) {
                                    peer
                                } else if let Some(peer) = client_peers.get_mut(&peer_id) {
                                    peer
                                } else {
                                    error!("[Peers] Peer lists is out-of-sync. This should never happen.");
                                    continue;
                                }
                            };

                            peer.set_state(PeerState::Connected);

                            // notify user
                            net_pub.send(Event::PeerConnected { peer_id, num_conns, timestamp }).await;
                        },
                        Event::SendRecvStopped { peer_id } => {

                            tcp_conns.remove(&peer_id);
                            udp_conns.remove(&peer_id);

                            // try to reconnect to server peers
                            if let Some(peer) = server_peers.get_mut(&peer_id) {

                                peer.set_state(PeerState::NotConnected);

                                raise_event_after_delay(Event::TryConnect {
                                    peer_id,
                                    retry: Some(RECONNECT_ATTEMPTS)
                                }, RECONNECT_COOLDOWN, &event_pub);

                            } else if client_peers.contains(&peer_id) {

                                client_peers.remove(&peer_id);

                            } else {
                                error!("[Peers] Peer lists is out-of-sync. This should never happen.");
                                continue;
                            }

                            let num_conns = tcp_conns.len() + udp_conns.len();

                            event_pub.send(Event::PeerDisconnected { peer_id, num_conns }).await;

                        },
                        Event::PeerDisconnected { peer_id, num_conns } => {

                            // notify user
                            net_pub.send(Event::PeerDisconnected { peer_id, num_conns }).await;
                        }
                        Event::PeerStalled { peer_id } => {

                            let mut peer = {
                                if let Some(peer) = server_peers.get_mut(&peer_id) {
                                    peer
                                } else if let Some(peer) = client_peers.get_mut(&peer_id) {
                                    peer
                                } else {
                                    error!("[Peers] Peer lists is out-of-sync. This should never happen.");
                                    continue;
                                }
                            };

                            peer.set_state(PeerState::Stalled);

                            // notify user
                            net_pub.send(Event::PeerStalled { peer_id }).await;
                        },
                        Event::BytesSent { to_peer, num_bytes, .. } => {},
                        Event::BytesBroadcasted { num_bytes, num_conns } => {}
                        Event::BytesReceived { from_peer, with_addr, num_bytes, buffer } => {

                            // notify user
                            net_pub.send(Event::BytesReceived { from_peer, with_addr, num_bytes, buffer }).await;
                        },
                        Event::TryConnect { peer_id, mut retry } => {

                            let mut peer = {
                                if let Some(peer) = server_peers.get_mut(&peer_id) {
                                    peer
                                } else if let Some(peer) = client_peers.get_mut(&peer_id) {
                                    peer
                                } else {
                                    error!("[Peers] Peer lists is out-of-sync. This should never happen.");
                                    continue;
                                }
                            };

                            // NOTE: this event happens after a certain time interval, so once it is raised
                            // the peer might already be connected.
                            if peer.is_connected() {
                                continue;
                            }

                            use Url::*;
                            match peer.url() {
                                Tcp(peer_addr) => {

                                    if !tcp::connect(&peer.id(), peer_addr, event_pub.clone()).await {

                                        let retry = match retry {
                                            Some(0) => Some(0),
                                            Some(1) => None,
                                            Some(n) => Some(n-1),
                                            None => continue,
                                        };

                                        info!("[Peers] Connection attempt failed. Retrying in {} ms", RECONNECT_COOLDOWN);

                                        raise_event_after_delay(Event::TryConnect { peer_id, retry }, RECONNECT_COOLDOWN, &event_pub);

                                    }
                                },
                                Udp(peer_addr) => {
                                    // TODO
                                },
                                _ => (),
                            }
                        },
                    }
                }
            }
        }
        */

        debug!("[Peers] Stopping actor");
    }
}

fn raise_event_after_delay(event: Event, after: u64, event_pub: &EventPub) {
    let mut event_pub = event_pub.clone();

    // finished once it has waited and send the event
    spawn(async move {
        task::sleep(Duration::from_millis(after)).await;

        event_pub
            .send(event)
            .await
            .expect("[Peers] Error sending event after delay");
    });
}

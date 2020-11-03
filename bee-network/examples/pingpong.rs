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

//! This example shows how to create and run 2 TCP nodes using `bee_network`, that will
//! automatically add eachother as peers and exchange the messages 'ping' and 'pong'
//! respectively.
//!
//! You might want to run several instances of such a node in separate
//! terminals and connect those instances by specifying commandline arguments.
//!
//! ```bash
//! cargo r --example pingpong -- --bind /ip4/127.0.0.1/tcp/1337 --peers /ip4/127.0.0.1/tcp/1338 --msg ping
//! cargo r --example pingpong -- --bind /ip4/127.0.0.1/tcp/1338 --peers /ip4/127.0.0.1/tcp/1337 --msg pong
//! ```

#![allow(dead_code, unused_imports)]

mod common;

use common::*;

use bee_common_ext::shutdown_tokio::Shutdown;
use bee_network::{Command::*, Event, EventReceiver, Multiaddr, Network, NetworkConfig, PeerId};

use futures::{
    channel::oneshot,
    select,
    sink::SinkExt,
    stream::{Fuse, StreamExt},
    AsyncWriteExt, FutureExt,
};
use log::*;
use structopt::StructOpt;

use std::{
    collections::{HashMap, HashSet},
    time::Duration,
};

const RECONNECT_INTERVAL: u64 = 5; // 5 seconds

#[tokio::main]
async fn main() {
    let args = Args::from_args();
    let config = args.into_config();

    logger::init(log::LevelFilter::Trace);

    let node = Node::builder(config).finish().await;

    let mut network = node.network.clone();
    let config = node.config.clone();

    info!("[pingpong] Adding static peers...");

    for peer_address in &config.peers {
        network
            .send(AddPeer {
                peer_address: peer_address.clone(),
            })
            .await
            .unwrap();
    }

    info!("[pingpong] ...finished.");

    node.run().await;
}

struct Node {
    config: Config,
    network: Network,
    events: flume::r#async::RecvStream<'static, Event>,
    peers: HashSet<PeerId>,
    handshakes: HashMap<String, Vec<PeerId>>,
}

impl Node {
    async fn run(self) {
        let Node {
            config,
            mut network,
            mut events,
            mut peers,
            mut handshakes,
            ..
        } = self;

        info!("[pingpong] Node running.");

        let mut ctrl_c = ctrl_c_listener().fuse();

        loop {
            select! {
                _ = ctrl_c => {
                    break;
                },
                event = events.next() => {
                    if let Some(event) = event {
                        info!("Received {}.", event);

                        process_event(event, &config.message, &mut network, &mut peers, &mut handshakes).await;
                    }
                },
            }
        }

        info!("[pingpong] Stopping node...");
        info!("[pingpong] Shutdown complete.");
    }

    pub fn builder(config: Config) -> NodeBuilder {
        NodeBuilder { config }
    }
}

#[inline]
async fn process_event(
    event: Event,
    message: &str,
    network: &mut Network,
    peers: &mut HashSet<PeerId>,
    handshakes: &mut HashMap<String, Vec<PeerId>>,
) {
    match event {
        Event::PeerAdded { peer_address } => {
            info!("[pingpong] Added peer {}.", peer_address);

            // network
            //     .send(ConnectPeer { peer_address })
            //     .await
            //     .expect("error sending Connect command");
        }

        Event::PeerRemoved {
            peer_address, peer_id, ..
        } => {
            info!("[pingpong] Removed peer {} ({:?}).", peer_address, peer_id);
        }

        Event::PeerConnected {
            peer_address,
            peer_id,
            origin,
            ..
        } => {
            info!("[pingpong] Connected peer {} ({}) [{}].", peer_address, peer_id, origin);

            // let message = Utf8Message::new(message);

            // network
            //     .send(SendMessage {
            //         peer_id: epid,
            //         message: message.as_bytes(),
            //     })
            //     .await
            //     .expect("error sending message to peer");
        }

        Event::PeerDisconnected { peer_id } => {
            info!("[pingpong] Disconnected peer {}.", peer_id);

            // endpoints.remove(&epid);

            // // TODO: remove epid from self.handshakes
            // // handshakes.remove(???);
        }

        Event::MessageReceived { peer_id, message, .. } => {
            info!("[pingpong] Received message from {} (len={}).", peer_id, message.len());

            // if !endpoints.contains(&epid) {
            //     // NOTE: first message is assumed to be the handshake message
            //     let handshake = Utf8Message::from_bytes(&message);
            //     info!("[pingpong] Received handshake '{}' ({})", handshake, epid);

            //     let epids = handshakes.entry(handshake.to_string()).or_insert_with(Vec::new);
            //     if !epids.contains(&epid) {
            //         epids.push(epid);
            //     }

            //     if epids.len() > 1 {
            //         info!(
            //             "[pingpong] '{0}' and '{1}' are duplicate connections. Dropping '{1}'...",
            //             epids[0], epids[1]
            //         );

            //         network
            //             .send(MarkDuplicate {
            //                 duplicate_epid: epids[1],
            //                 original_epid: epids[0],
            //             })
            //             .await
            //             .expect("error sending 'MarkDuplicate'");

            //         network
            //             .send(DisconnectEndpoint { epid: epids[1] })
            //             .await
            //             .expect("error sending 'DisconnectEndpoint' command");
            //     }

            //     endpoints.insert(epid);

            //     spam_endpoint(network.clone(), epid);
            // } else {
            //     let message = Utf8Message::from_bytes(&message);
            //     info!("[pingpong] Received message '{}' ({})", message, epid);
            // }
        }

        _ => warn!("Unsupported event {}.", event),
    }
}

fn ctrl_c_listener() -> oneshot::Receiver<()> {
    let (sender, receiver) = oneshot::channel();

    tokio::spawn(async move {
        tokio::signal::ctrl_c().await.unwrap();

        sender.send(()).unwrap();
    });

    receiver
}

fn spam_endpoint(mut network: Network, peer_id: PeerId) {
    info!("[pingpong] Now sending spam messages to {}", peer_id);

    tokio::spawn(async move {
        for i in 0u64.. {
            tokio::time::delay_for(Duration::from_secs(5)).await;

            let message = Utf8Message::new(&i.to_string());

            network
                .send(SendMessage {
                    peer_id: peer_id.clone(),
                    message: message.as_bytes(),
                })
                .await
                .expect("error sending number");
        }
    });
}

struct NodeBuilder {
    config: Config,
}

impl NodeBuilder {
    pub async fn finish(self) -> Node {
        let network_config = NetworkConfig::builder()
            .bind_address(&self.config.bind_address.to_string())
            .reconnect_interval(RECONNECT_INTERVAL)
            .finish();

        let mut shutdown = Shutdown::new();

        info!("[pingpong] Initializing network...");
        let (network, events) = bee_network::init(network_config, &mut shutdown).await;

        info!("[pingpong] Node initialized.");
        Node {
            config: self.config,
            network,
            events: events.into_stream(),
            peers: HashSet::new(),
            handshakes: HashMap::new(),
        }
    }
}

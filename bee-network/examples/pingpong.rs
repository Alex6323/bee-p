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
use bee_network::{Command::*, Event, EventReceiver, Keypair, Multiaddr, Network, NetworkConfig, PeerId};

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

const RECONNECT_MILLIS: u64 = 5000;

#[tokio::main]
async fn main() {
    let args = Args::from_args();
    let config = args.into_config();

    logger::init(log::LevelFilter::Trace);

    let node = Node::builder(config).finish().await;

    let mut network = node.network.clone();
    let config = node.config.clone();

    info!("[EXAMPLE] Adding static peers...");

    // for (peer_address, peer_id) in &config.peers {
    for peer_address in &config.peers {
        network
            .send(ConnectUnknownPeer {
                address: peer_address.clone(),
            })
            .await
            .unwrap();
    }

    info!("[EXAMPLE] ...finished.");

    node.run().await;
}

struct Node {
    config: Config,
    network: Network,
    events: flume::r#async::RecvStream<'static, Event>,
    peers: HashSet<PeerId>,
    handshakes: HashMap<String, Vec<PeerId>>,
    shutdown: Shutdown,
}

impl Node {
    async fn run(self) {
        let Node {
            config,
            mut network,
            mut events,
            mut peers,
            mut handshakes,
            shutdown,
        } = self;

        info!("[EXAMPLE] Node running.");

        let mut ctrl_c = ctrl_c_listener().fuse();

        loop {
            select! {
                _ = ctrl_c => {
                    break;
                },
                event = events.next() => {
                    if let Some(event) = event {
                        info!("Received {:?}.", event);

                        process_event(event, &config.message, &mut network, &mut peers, &mut handshakes).await;
                    }
                },
            }
        }

        info!("[EXAMPLE] Stopping node...");
        shutdown.execute().await.expect("shutdown error");

        info!("[EXAMPLE] Shutdown complete.");
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
        Event::PeerConnected {
            id, address, origin, ..
        } => {
            info!(
                "[EXAMPLE] Connected peer '{}' with address '{}' [{}].",
                id, address, origin
            );

            info!("[EXAMPLE] Sending message: \"{}\"", message);
            network
                .send(SendMessage {
                    message: Utf8Message::new(message).as_bytes(),
                    to: id.clone(),
                })
                .await
                .expect("error sending message to peer");

            spam_endpoint(network.clone(), id);
        }

        Event::PeerDisconnected { id } => {
            info!("[EXAMPLE] Disconnected peer {}.", id);
        }

        Event::MessageReceived { message, from } => {
            info!("[EXAMPLE] Received message from {} (length: {}).", from, message.len());

            let message = Utf8Message::from_bytes(&message);
            info!("[EXAMPLE] Received message \"{}\"", message);
        }

        _ => warn!("Unsupported event {:?}.", event),
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
    info!("[EXAMPLE] Now sending spam messages to {}", peer_id);

    tokio::spawn(async move {
        for i in 0u64.. {
            tokio::time::delay_for(Duration::from_secs(5)).await;

            let message = Utf8Message::new(&i.to_string());

            network
                .send(SendMessage {
                    message: message.as_bytes(),
                    to: peer_id.clone(),
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
        let network_config = NetworkConfig::build()
            .bind_address(&self.config.bind_address.to_string())
            .reconnect_millis(RECONNECT_MILLIS)
            .finish();

        let mut shutdown = Shutdown::new();

        info!("[EXAMPLE] Initializing network...");
        let local_keys = Keypair::generate();
        let (network, events) = bee_network::init(network_config, local_keys, &mut shutdown).await;

        info!("[EXAMPLE] Node initialized.");
        Node {
            config: self.config,
            network,
            events: events.into_stream(),
            peers: HashSet::new(),
            handshakes: HashMap::new(),
            shutdown,
        }
    }
}

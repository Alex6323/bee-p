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
//! cargo r --example pingpong -- --bind localhost:1337 --peers tcp://localhost:1338 --msg ping
//! cargo r --example pingpong -- --bind localhost:1338 --peers tcp://localhost:1337 --msg pong
//! ```

#![allow(dead_code, unused_imports)]

mod common;

use bee_common_ext::shutdown_tokio::Shutdown;
use bee_network::{Command::*, EndpointId, Event, Events, Network, NetworkConfig, Origin};

use common::*;

use futures::{
    channel::{mpsc, oneshot},
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
    let config = args.config();

    logger::init(log::LevelFilter::Debug);

    let node = Node::builder(config).finish().await;

    let mut network = node.network.clone();
    let config = node.config.clone();

    info!("Adding static peers...");
    // tokio::time::delay_for(Duration::from_secs(1)).await; // remove this?
    for url in &config.peers {
        network.send(AddEndpoint { url: url.clone() }).await.unwrap();
    }

    info!("Finished.");

    node.run().await;
}

struct Node {
    pub config: Config,
    pub network: Network,
    pub events: Events,
    shutdown: Shutdown,
    pub handshakes: HashMap<String, Vec<EndpointId>>,
    pub endpoints: HashSet<EndpointId>,
}

impl Node {
    async fn run(self) {
        let Node {
            config,
            mut network,
            mut events,
            ..
        } = self;

        info!("Node running.");

        let mut ctrl_c = ctrl_c_listener().fuse();

        loop {
            select! {
                _ = ctrl_c => {
                    break;
                },
                event = events.next() => {
                    if let Some(event) = event {
                        info!("Received {}.", event);

                        process_event(event, &config.message, &mut network).await;
                    }
                },
            }
        }

        info!("Stopping node...");

        self.shutdown.execute().await.expect("error shutting down gracefully.");

        info!("Shutdown complete.");
    }

    pub fn builder(config: Config) -> NodeBuilder {
        NodeBuilder { config }
    }
}

#[inline]
async fn process_event(event: Event, message: &String, network: &mut Network) {
    match event {
        Event::EndpointAdded { epid } => {
            info!("Added endpoint {}.", epid);

            network
                .send(ConnectEndpoint { epid })
                .await
                .expect("error sending Connect command");
        }

        // Event::EndpointRemoved { epid, .. } => {
        //     info!("Removed endpoint {}.", epid);
        // }

        // Event::EndpointConnected { epid, origin, .. } => {
        //     info!("Connected endpoint {} ({}).", epid, origin);

        //     let utf8_message = Utf8Message::new(message);

        //     network
        //         .send(SendMessage {
        //             receiver_epid,
        //             message: utf8_message.as_bytes(),
        //         })
        //         .await
        //         .expect("error sending message to peer");
        // }

        // Event::EndpointDisconnected { epid, .. } => {
        //     info!("Disconnected endpoint {}.", epid);

        //     self.endpoints.remove(&epid);

        //     // TODO: remove epid from self.handshakes
        // }

        // Event::MessageReceived { epid, message, .. } => {
        //     if !self.endpoints.contains(&epid) {
        //         let handshake = Utf8Message::from_bytes(&message);
        //         info!("Received handshake '{}' ({})", handshake, epid);

        //         let epids = self.handshakes.entry(handshake.to_string()).or_insert(Vec::new());
        //         if !epids.contains(&epid) {
        //             epids.push(epid);
        //         }

        //         if epids.len() > 1 {
        //             info!("'{}' and '{}' are duplicate connections.", epids[0], epids[1]);

        //             self.network
        //                 .send(SetDuplicate {
        //                     epid: epids[0],
        //                     other: epids[1],
        //                 })
        //                 .await
        //                 .expect("error sending 'Disconnect'");
        //         }

        //         self.endpoints.insert(epid);
        //     } else {
        //         let message = Utf8Message::from_bytes(&message);
        //         info!("Received message '{}' ({})", message, epid);
        //     }

        //     // TODO: send the next message

        //     // let utf8_message = Utf8Message::new(&self.message);

        //     // self.network
        //     //     .send(SendMessage {
        //     //         epid,
        //     //         bytes: utf8_message.as_bytes(),
        //     //     })
        //     //     .await
        //     //     .expect("error sending message to peer");
        // }
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

// fn spam(mut network: Network, msg: Utf8Message, num: usize, interval: u64) {
//     info!("Sending {:?} messages", num);

//     task::block_on(async move {
//         for _ in 0..num {
//             task::sleep(std::time::Duration::from_millis(interval)).await;
//             network
//                 .send(SendMessage { bytes: msg.as_bytes() })
//                 .await
//                 .expect("error broadcasting bytes");
//         }
//     });
// }

struct NodeBuilder {
    config: Config,
}

impl NodeBuilder {
    pub async fn finish(self) -> Node {
        let mut shutdown = Shutdown::new();

        let network_config = NetworkConfig::builder()
            .binding_address(&self.config.binding_address.ip().to_string())
            .binding_port(self.config.binding_address.port())
            .reconnect_interval(RECONNECT_INTERVAL)
            .finish();

        info!("Initializing network...");
        let (network, events) = bee_network::init(network_config, &mut shutdown).await;

        info!("Node initialized.");
        Node {
            config: self.config,
            network,
            events,
            shutdown,
            handshakes: HashMap::new(),
            endpoints: HashSet::new(),
        }
    }
}

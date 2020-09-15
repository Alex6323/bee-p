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

use bee_common::shutdown::Shutdown;
use bee_network::{Command::*, EndpointId, Event, Events, Network, NetworkConfig, Origin, Url};

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

// use std::io::Write;

use async_std::io::stdout;

use std::collections::{HashMap, HashSet};

mod common;

const RECONNECT_INTERVAL: u64 = 5; // 5 seconds

#[tokio::main]
fn main() {
    let args = Args::from_args();
    let config = args.config();

    logger::init(log::LevelFilter::Debug);

    let mut node = Node::builder(config, args.msg).finish();

    info!("Node initialized.");

    let mut network = node.network.clone();
    let config = node.config.clone();

    info!("Adding static peers...");
    block_on(async {
        // FIXME: Not waiting here a little can cause a deadlock (sometimes).
        task::sleep(std::time::Duration::from_millis(5)).await;

        for url in &config.peers {
            network.send(AddEndpoint { url: url.clone() }).await.unwrap();
        }
    });

    node.run_loop();
    node.shutdown();
}

struct Node {
    pub config: Config,
    pub network: Network,
    pub events: Events,
    shutdown: Shutdown,
    pub message: String,
    pub handshakes: HashMap<String, Vec<EndpointId>>,
    pub endpoints: HashSet<EndpointId>,
}

impl Node {
    fn run_loop(&mut self) {
        info!("Node running.");

        // let mut ctrl_c = ctrl_c_listener().fuse();

        block_on(async {
            loop {
                select! {
                    // _ = ctrl_c => {
                    //     break;
                    // },
                    event = self.events.next() => {
                        if let Some(event) = event {
                            info!("Received {}.", event);

                            self.handle_event(event).await;
                        }
                    },
                }
            }
        });
    }

    // #[inline]
    async fn handle_event(&mut self, event: Event) {
        match event {
            Event::EndpointAdded { epid, total } => {
                info!("Added endpoint {} ({}).", epid, total);

                self.network
                    .send(Connect { epid })
                    .await
                    .expect("error sending Connect command");
            }

            Event::EndpointRemoved { epid, .. } => {
                info!("Removed endpoint {}.", epid);
            }

            Event::EndpointConnected { epid, origin, .. } => {
                info!("Connected endpoint {} ({}).", epid, origin);

                let utf8_message = Utf8Message::new(&self.message);

                self.network
                    .send(SendMessage {
                        epid,
                        message: utf8_message.as_bytes(),
                    })
                    .await
                    .expect("error sending message to peer");
            }

            Event::EndpointDisconnected { epid, .. } => {
                info!("Disconnected endpoint {}.", epid);

                self.endpoints.remove(&epid);

                // TODO: remove epid from self.handshakes
            }

            Event::MessageReceived { epid, message, .. } => {
                if !self.endpoints.contains(&epid) {
                    let handshake = Utf8Message::from_bytes(&message);
                    info!("Received handshake '{}' ({})", handshake, epid);

                    let epids = self.handshakes.entry(handshake.to_string()).or_insert(Vec::new());
                    if !epids.contains(&epid) {
                        epids.push(epid);
                    }

                    if epids.len() > 1 {
                        info!("'{}' and '{}' are duplicate connections.", epids[0], epids[1]);

                        self.network
                            .send(SetDuplicate {
                                epid: epids[0],
                                other: epids[1],
                            })
                            .await
                            .expect("error sending 'Disconnect'");
                    }

                    self.endpoints.insert(epid);
                } else {
                    let message = Utf8Message::from_bytes(&message);
                    info!("Received message '{}' ({})", message, epid);
                }

                // TODO: send the next message

                // let utf8_message = Utf8Message::new(&self.message);

                // self.network
                //     .send(SendMessage {
                //         epid,
                //         bytes: utf8_message.as_bytes(),
                //     })
                //     .await
                //     .expect("error sending message to peer");
            }
            _ => warn!("Unsupported event {}.", event),
        }
    }

    fn shutdown(self) {
        info!("Stopping node...");

        block_on(self.shutdown.execute()).expect("error shutting down");

        info!("Shutdown complete.");
    }

    pub fn builder(config: Config, message: String) -> NodeBuilder {
        NodeBuilder { config, message }
    }
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
    message: String,
}

impl NodeBuilder {
    pub fn finish(self) -> Node {
        let mut shutdown = Shutdown::new();

        info!("Initializing network...");
        let network_config = NetworkConfig::builder()
            .binding_addr(&self.config.host_addr.ip().to_string())
            .binding_port(*self.config.host_addr.port())
            .reconnect_interval(RECONNECT_INTERVAL)
            .finish();

        let (network, events) = bee_network::init(network_config, &mut shutdown);

        Node {
            config: self.config,
            network,
            events,
            shutdown,
            message: self.message,
            handshakes: HashMap::new(),
            endpoints: HashSet::new(),
        }
    }
}

// fn ctrl_c_listener() -> oneshot::Receiver<()> {
//     let (sender, receiver) = oneshot::channel();

//     spawn(async move {
//         let mut tokio = tokio::runtime::Runtime::new().expect("Error creating Tokio runtime.");

//         tokio
//             .block_on(tokio::signal::ctrl_c())
//             .expect("Error blocking on CTRL-C.");

//         sender.send(()).expect("Error sending shutdown signal.");
//     });

//     receiver
// }

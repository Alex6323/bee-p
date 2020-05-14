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

use bee_network::{
    Command::*, EndpointId as EpId, Event, EventSubscriber as Events, Network, NetworkConfig, NetworkConfigBuilder,
    Origin, Shutdown, Url,
};

use common::*;

use async_std::task::{self, block_on};
use futures::prelude::*;
use log::*;
use structopt::StructOpt;

mod common;

fn main() {
    let args = Args::from_args();
    let config = args.make_config();

    logger::init(log::LevelFilter::Info);

    let (network, shutdown, events) = bee_network::init(NetworkConfigBuilder::new().build());

    let mut node = Node::builder()
        .with_network(network.clone())
        .with_shutdown(shutdown)
        .build();

    task::spawn(notification_handler(events, network, args.msg));

    block_on(node.init(config));
    block_on(node.shutdown());
}

async fn notification_handler(mut events: Events, mut network: Network, msg: String) {
    let network = &mut network;

    while let Some(event) = events.next().await {
        info!("[.....] Received {}.", event);

        match event {
            Event::EndpointAdded { epid, total } => {
                info!("[.....] Added endpoint {} ({}).", epid, total);

                network
                    .send(Connect { epid, responder: None })
                    .await
                    .expect("error sending Connect command");
            }
            Event::EndpointConnected { epid, origin, .. } => {
                info!("[.....] Connected endpoint {} ({}).", epid, origin);

                let msg = Utf8Message::new(&msg);
                network
                    .send(SendMessage {
                        epid,
                        bytes: msg.as_bytes(),
                        responder: None,
                    })
                    .await
                    .expect("error sending SendMessage command");
            }
            Event::MessageReceived { epid, bytes, .. } => {
                info!(
                    "[.....] Received message '{}' ({})",
                    Utf8Message::from_bytes(&bytes),
                    epid
                );
            }
            _ => (),
        }
    }
}

struct Node {
    network: Network,
    shutdown: Shutdown,
}

impl Node {
    pub async fn init(&mut self, config: Config) {
        info!("[.....] Initializing...");

        for peer in config.peers {
            self.add_peer(peer.clone()).await;
        }

        info!("[.....] Initialized.");
    }

    pub async fn add_peer(&mut self, url: Url) {
        self.network.send(AddEndpoint { url, responder: None }).await.unwrap();
    }

    pub async fn shutdown(self) {
        self.block_on_ctrl_c();

        info!("[.....] Shutting down...");

        self.shutdown.execute().await;

        info!("[.....] Shutdown complete. See you soon!");
    }

    fn block_on_ctrl_c(&self) {
        let mut rt = tokio::runtime::Runtime::new().expect("[Node ] Error creating Tokio runtime");

        rt.block_on(tokio::signal::ctrl_c())
            .expect("[Node ] Error blocking on CTRL-C");
    }

    pub fn builder() -> NodeBuilder {
        NodeBuilder::new()
    }
}

fn spam(mut network: Network, msg: Utf8Message, num: usize, interval: u64) {
    info!("[Expl ] Starting spammer: {:?} messages", num);

    task::block_on(async move {
        for _ in 0..num {
            task::sleep(std::time::Duration::from_millis(interval)).await;
            network
                .send(BroadcastMessage {
                    bytes: msg.as_bytes(),
                    responder: None,
                })
                .await
                .expect("error broadcasting bytes");
        }
    });

    info!("[Expl ] Spammer stopped.");
}

struct NodeBuilder {
    network: Option<Network>,
    shutdown: Option<Shutdown>,
}

impl NodeBuilder {
    pub fn new() -> Self {
        Self {
            network: None,
            shutdown: None,
        }
    }

    pub fn with_network(mut self, network: Network) -> Self {
        self.network.replace(network);
        self
    }

    pub fn with_shutdown(mut self, shutdown: Shutdown) -> Self {
        self.shutdown.replace(shutdown);
        self
    }

    pub fn build(self) -> Node {
        Node {
            network: self.network.expect("[Node ] No network instance provided"),
            shutdown: self.shutdown.expect("[Node ] No shutdown instance provided"),
        }
    }
}

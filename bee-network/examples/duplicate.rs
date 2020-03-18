//! This example shows how two TCP nodes follow a predefined policy to drop a duplicate
//! connection, and end up agreeing on one single TCP connection between them.
//!
//! You might want to run several instances of such a node in separate
//! terminals and connect those instances by specifying commandline arguments.
//!
//! ```bash
//! cargo r --example duplicate -- --bind localhost:1337 --peers tcp://localhost:1338 --msg ping
//! cargo r --example duplicate -- --bind localhost:1338 --peers tcp://localhost:1337 --msg pong
//! ```

#![recursion_limit = "256"]

use netzwerk::{
    Command::*,
    Config,
    Event,
    EventSubscriber as Events,
    Network,
    Peer,
    PeerId,
    Shutdown,
};

use common::*;

use async_std::prelude::*;
use async_std::task::{
    self,
    block_on,
    spawn,
};
use futures::channel::oneshot;
use futures::{
    select,
    FutureExt,
};
use log::*;
use serde::{
    Deserialize,
    Serialize,
};
use structopt::StructOpt;

mod common;

fn main() {
    let args = Args::from_args();
    let config = args.make_config();

    logger::init(log::LevelFilter::Info);

    let mut node = Node::from_config(config.clone());

    block_on(node.init());

    block_on(node.wait());
    block_on(node.exit());
}

enum Message {
    Text(Utf8Message),
    Handshake(Handshake),
}

#[derive(Serialize, Deserialize, Debug)]
struct Handshake {
    server_port: u16,
}

impl Handshake {
    pub fn serialize(&self) -> Vec<u8> {
        bincode::serialize(self).unwrap()
    }
}

enum NodeState {
    Off,
    Initializing,
    Running,
    Stopping,
}

struct Node {
    config: Config,
    network: Option<Network>,
    shutdown: Option<Shutdown>,
    state: NodeState,
    receiver: Option<oneshot::Receiver<()>>,
}

impl Node {
    pub fn from_config(config: Config) -> Self {
        Self {
            config,
            network: None,
            shutdown: None,
            state: NodeState::Off,
            receiver: None,
        }
    }

    pub async fn init(&mut self) {
        self.state = NodeState::Initializing;

        let (network, shutdown, events) = netzwerk::init(self.config.clone());

        self.network.replace(network.clone());
        self.shutdown.replace(shutdown);

        for peer in self.config.peers().values() {
            self.add_peer(peer.clone()).await;
        }

        let (sender, receiver) = oneshot::channel();

        spawn(event_loop(events, sender, network, self.config.clone()));

        self.receiver.replace(receiver);

        info!("[Node ] Running...");
        self.state = NodeState::Running;
    }

    pub async fn add_peer(&mut self, peer: Peer) {
        if let Some(network) = self.network.as_mut() {
            network
                .send(AddPeer {
                    peer,
                    connect_attempts: Some(5),
                })
                .await;
        }
    }

    pub async fn remove_peer(&mut self, peer_id: PeerId) {
        if let Some(network) = self.network.as_mut() {
            network.send(RemovePeer { peer_id }).await;
        }
    }

    pub async fn connect(&mut self, peer_id: PeerId) {
        if let Some(network) = self.network.as_mut() {
            network
                .send(Connect {
                    peer_id,
                    num_retries: 5,
                })
                .await;
        }
    }

    pub async fn send_handshake(&mut self, handshake: Handshake, peer_id: PeerId) {
        if let Some(network) = self.network.as_mut() {
            network
                .send(SendBytes {
                    to_peer: peer_id,
                    bytes: handshake.serialize(),
                })
                .await;
        }
    }

    pub async fn wait(&mut self) {
        if let Some(receiver) = self.receiver.as_mut() {
            receiver.await.expect("error receiving shutdown signal");
        }
    }

    pub async fn exit(mut self) {
        info!("[Node ] Shutting down...");
        self.state = NodeState::Stopping;

        if let Some(mut network) = self.network {
            network.send(Shutdown).await;
        }
        if let Some(mut shutdown) = self.shutdown {
            shutdown.finish_tasks().await;
        }

        info!("[Node ] Complete. See you soon!");
    }
}

async fn event_loop(mut events: Events, sender: oneshot::Sender<()>, mut network: Network, config: Config) {
    let el_shutdown = &mut shutdown_listener();

    loop {
        select! {
            event = events.next().fuse() => {
                let event = event.unwrap();
                info!("[Node ] {:?} received.", event);

                match event {
                    Event::PeerConnected { peer_id, num_conns, timestamp } => {

                        info!("[Node ] Sending handshake to {}", peer_id);
                        let handshake = Handshake { server_port: config.binding_addr.port().unwrap() };
                        network.send(SendBytes { to_peer: peer_id, bytes: handshake.serialize() }).await;
                    }
                    Event::BytesReceived { from_peer, num_bytes, buffer, .. } => {
                        info!("[Node ] Received: '{}' bytes from peer {}", num_bytes, from_peer);
                            let handshake: Handshake = bincode::deserialize(&buffer[0..num_bytes])
                                .expect("error deserializing handshake");

                        info!("[Node ] Handshake: {:?}", handshake);
                    }
                    _ => (),
                }
            },
            shutdown = el_shutdown.fuse() => {
                break;
            }
        }
    }

    sender.send(()).expect("error sending shutdown signal");
}

fn shutdown_listener() -> oneshot::Receiver<()> {
    let (sender, receiver) = oneshot::channel();

    spawn(async move {
        let mut rt = tokio::runtime::Runtime::new().expect("[Node ] Error creating Tokio runtime");

        rt.block_on(tokio::signal::ctrl_c())
            .expect("[Node ] Error blocking on CTRL-C");

        sender.send(()).expect("error sending shutdown signal");
    });

    receiver
}

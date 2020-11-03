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

//#![warn(missing_docs)]

pub use config::{NetworkConfig, NetworkConfigBuilder};
pub use interaction::{
    commands::{self, Command},
    events::{self, Event, EventReceiver},
};

pub use network::Network;

mod config;
mod conns;
mod interaction;
mod network;
mod peers;
mod transport;

use config::{DEFAULT_MAX_BUFFER_SIZE, DEFAULT_RECONNECT_INTERVAL};
use conns::ConnectionManager;
use peers::{KnownPeerList, PeerManager};

use bee_common_ext::shutdown_tokio::Shutdown;

use futures::channel::oneshot;
use libp2p::identity;
#[doc(inline)]
pub use libp2p::{Multiaddr, PeerId};
use log::*;

use std::{
    fs::File,
    sync::atomic::{AtomicU64, AtomicUsize, Ordering},
};

pub(crate) static MAX_BUFFER_SIZE: AtomicUsize = AtomicUsize::new(DEFAULT_MAX_BUFFER_SIZE);
pub(crate) static RECONNECT_INTERVAL: AtomicU64 = AtomicU64::new(DEFAULT_RECONNECT_INTERVAL);

pub async fn init(config: NetworkConfig, shutdown: &mut Shutdown) -> (Network, EventReceiver) {
    // TODO: Restore keys from fs.
    let local_keys = identity::Keypair::generate_ed25519();
    let local_peer = PeerId::from_public_key(local_keys.public());
    trace!("local peer id = {}", local_peer);

    let (command_sender, command_receiver) = commands::channel();
    let (event_sender, event_receiver) = events::channel();
    let (internal_event_sender, internal_event_receiver) = events::channel();

    let (peer_manager_shutdown_sender, peer_manager_shutdown_receiver) = oneshot::channel();
    let (conn_manager_shutdown_sender, conn_manager_shutdown_receiver) = oneshot::channel();

    let known_peers = KnownPeerList::new();

    let peer_manager = PeerManager::new(
        local_keys.clone(),
        command_receiver,
        event_sender,
        internal_event_receiver,
        internal_event_sender.clone(),
        known_peers.clone(),
        peer_manager_shutdown_receiver,
    );

    let conn_manager = ConnectionManager::new(
        local_keys,
        config.bind_address.clone(),
        internal_event_sender,
        conn_manager_shutdown_receiver,
        known_peers,
    );

    let peer_manager_task = tokio::spawn(peer_manager.run());
    let conn_manager_task = tokio::spawn(conn_manager.run());

    shutdown.add_worker_shutdown(peer_manager_shutdown_sender, peer_manager_task);
    shutdown.add_worker_shutdown(conn_manager_shutdown_sender, conn_manager_task);

    MAX_BUFFER_SIZE.swap(config.max_buffer_size, Ordering::Relaxed);
    RECONNECT_INTERVAL.swap(config.reconnect_interval, Ordering::Relaxed);

    (Network::new(config, command_sender), event_receiver)
}

const ID_KEYS_FILEPATH: &str = "./node_id.txt";

fn load_or_create_identity() -> PeerId {
    //
    if let Err(e) = File::open(ID_KEYS_FILEPATH) {
        create_new_identity()
    } else {
        load_identity()
    }
}

fn create_new_identity() -> PeerId {
    todo!();
}

fn load_identity() -> PeerId {
    // let mut file = File::create("node_id.txt")?;
    // file.write_all(b"Hello, world!")?;
    todo!()
}

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

// #![warn(missing_docs)]
#![recursion_limit = "512"]

mod config;
mod conns;
mod interaction;
mod multiaddr_ext;
mod network;
mod peers;
mod protocols;
mod transport;

// Exports
pub use config::{NetworkConfig, NetworkConfigBuilder};
pub use conns::Origin;
pub use interaction::{
    commands::{self, Command},
    events::{self, Event},
};
#[doc(inline)]
pub use libp2p::{core::identity::ed25519::Keypair, Multiaddr, PeerId};
pub use multiaddr_ext::MultiaddrPeerId;
pub use network::Network;

pub type EventReceiver = flume::Receiver<Event>;

use config::{DEFAULT_MSG_BUFFER_SIZE, DEFAULT_PEER_LIMIT, DEFAULT_RECONNECT_MILLIS};
use conns::ConnectionManager;
use interaction::events::InternalEvent;
use peers::{BannedAddrList, BannedPeerList, PeerList, PeerManager};

use bee_common_ext::shutdown_tokio::Shutdown;

use futures::channel::oneshot;
use libp2p::identity;

use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};

pub(crate) static MSG_BUFFER_SIZE: AtomicUsize = AtomicUsize::new(DEFAULT_MSG_BUFFER_SIZE);
pub(crate) static RECONNECT_MILLIS: AtomicU64 = AtomicU64::new(DEFAULT_RECONNECT_MILLIS);
pub(crate) static PEER_LIMIT: AtomicUsize = AtomicUsize::new(DEFAULT_PEER_LIMIT);

pub async fn init(config: NetworkConfig, local_keys: Keypair, shutdown: &mut Shutdown) -> (Network, EventReceiver) {
    let local_keys = identity::Keypair::Ed25519(local_keys);
    let local_id = PeerId::from_public_key(local_keys.public());

    let (command_sender, command_receiver) = commands::channel();
    let (event_sender, event_receiver) = events::channel::<Event>();
    let (internal_event_sender, internal_event_receiver) = events::channel::<InternalEvent>();

    let (peer_manager_shutdown_sender, peer_manager_shutdown_receiver) = oneshot::channel();
    let (conn_manager_shutdown_sender, conn_manager_shutdown_receiver) = oneshot::channel();

    let banned_addrs = BannedAddrList::new();
    let banned_peers = BannedPeerList::new();
    let peers = PeerList::new();

    let peer_manager = PeerManager::new(
        local_keys.clone(),
        command_receiver,
        event_sender,
        internal_event_receiver,
        internal_event_sender.clone(),
        peers.clone(),
        banned_addrs.clone(),
        banned_peers.clone(),
        peer_manager_shutdown_receiver,
    );

    let conn_manager = ConnectionManager::new(
        local_keys,
        config.bind_address.clone(),
        internal_event_sender,
        conn_manager_shutdown_receiver,
        peers,
        banned_addrs,
        banned_peers,
    )
    .unwrap_or_else(|e| {
        panic!("Fatal error: {}", e);
    });

    let listen_address = conn_manager.listen_address.clone();

    let peer_manager_task = tokio::spawn(peer_manager.run());
    let conn_manager_task = tokio::spawn(conn_manager.run());

    shutdown.add_worker_shutdown(peer_manager_shutdown_sender, peer_manager_task);
    shutdown.add_worker_shutdown(conn_manager_shutdown_sender, conn_manager_task);

    MSG_BUFFER_SIZE.swap(config.msg_buffer_size, Ordering::Relaxed);
    RECONNECT_MILLIS.swap(config.reconnect_millis, Ordering::Relaxed);
    PEER_LIMIT.swap(config.peer_limit, Ordering::Relaxed);

    (
        Network::new(config, command_sender, listen_address, local_id),
        event_receiver,
    )
}

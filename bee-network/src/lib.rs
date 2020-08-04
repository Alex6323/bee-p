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

pub use address::{url::Url, Address, Port};
pub use commands::Command;
pub use config::{NetworkConfig, NetworkConfigBuilder};
pub use endpoint::{Endpoint, EndpointId};
pub use events::Event;
pub use tcp::connection::Origin;

pub use network::Network;

mod address;
mod commands;
mod config;
mod endpoint;
mod events;
mod network;
mod tcp;
mod utils;

use config::{DEFAULT_MAX_TCP_BUFFER_SIZE, DEFAULT_RECONNECT_INTERVAL};
use endpoint::{allowlist, worker::EndpointWorker};
use tcp::worker::TcpWorker;

use bee_common::shutdown::Shutdown;

use async_std::task::spawn;
use futures::{
    channel::{mpsc, oneshot},
    stream,
    stream::StreamExt,
};

use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};

pub(crate) static MAX_TCP_BUFFER_SIZE: AtomicUsize = AtomicUsize::new(DEFAULT_MAX_TCP_BUFFER_SIZE);
pub(crate) static RECONNECT_INTERVAL: AtomicU64 = AtomicU64::new(DEFAULT_RECONNECT_INTERVAL);

// NOTE: we make this an opaque type because it is exposed.
// pub struct Events(stream::Fuse<mpsc::Receiver<Event>>);
pub struct Events(mpsc::Receiver<Event>);

impl std::ops::Deref for Events {
    // type Target = stream::Fuse<mpsc::Receiver<Event>>;
    type Target = mpsc::Receiver<Event>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for Events {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

pub fn init(config: NetworkConfig, shutdown: &mut Shutdown) -> (Network, Events) {
    // Create communication channels.
    let (command_sender, command_receiver) = commands::channel();
    let (event_sender, event_receiver) = events::channel();
    let (internal_event_sender, internal_event_receiver) = events::channel();

    // Create channels to signal shutdown to the workers.
    let (endpoint_worker_shutdown_sender, endpoint_worker_shutdown_receiver) = oneshot::channel();
    let (tcp_worker_shutdown_sender, tcp_worker_shutdown_receiver) = oneshot::channel();

    // Create the worker that manages the endpoints to connect to.
    let endpoint_worker = EndpointWorker::new(
        command_receiver,
        event_sender,
        internal_event_receiver,
        internal_event_sender.clone(),
    );

    // Create the worker that manages the TCP connections established with the endpoints.
    let tcp_worker = TcpWorker::new(config.socket_addr(), internal_event_sender);

    // Spawn workers, and connect them to the shutdown mechanism.
    shutdown.add_worker_shutdown(
        endpoint_worker_shutdown_sender,
        // endpoint,
        spawn(endpoint_worker.run(endpoint_worker_shutdown_receiver)),
    );
    shutdown.add_worker_shutdown(
        tcp_worker_shutdown_sender,
        // tcp,
        spawn(tcp_worker.run(tcp_worker_shutdown_receiver)),
    );

    // Initialize Allowlist and make sure it gets dropped when the shutdown occurs.
    allowlist::init();
    shutdown.add_action(|| allowlist::drop());

    MAX_TCP_BUFFER_SIZE.swap(config.max_tcp_buffer_size, Ordering::Relaxed);
    RECONNECT_INTERVAL.swap(config.reconnect_interval, Ordering::Relaxed);

    // (Network::new(config, command_sender), Events(event_receiver.fuse()))
    (Network::new(config, command_sender), Events(event_receiver))
}

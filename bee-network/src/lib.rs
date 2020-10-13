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

pub use command::Command;
pub use config::{NetworkConfig, NetworkConfigBuilder};
pub use endpoint::EndpointId;
pub use event::{Event, EventReceiver};
pub use tcp::Origin;

pub use network::Network;

mod command;
mod config;
mod endpoint;
mod event;
mod network;
mod tcp;
mod util;

use config::{DEFAULT_MAX_TCP_BUFFER_SIZE, DEFAULT_RECONNECT_INTERVAL};
use endpoint::{EndpointContactList, EndpointWorker};
use tcp::TcpServer;

use bee_common_ext::shutdown_tokio::Shutdown;

use futures::channel::oneshot;

use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};

pub(crate) static MAX_TCP_BUFFER_SIZE: AtomicUsize = AtomicUsize::new(DEFAULT_MAX_TCP_BUFFER_SIZE);
pub(crate) static RECONNECT_INTERVAL: AtomicU64 = AtomicU64::new(DEFAULT_RECONNECT_INTERVAL);

pub async fn init(config: NetworkConfig, shutdown: &mut Shutdown) -> (Network, EventReceiver) {
    let (command_sender, command_receiver) = command::channel();
    let (event_sender, event_receiver) = event::channel();
    let (internal_event_sender, internal_event_receiver) = event::channel();
    let (endpoint_worker_shutdown_sender, endpoint_worker_shutdown_receiver) = oneshot::channel();
    let (tcp_server_shutdown_sender, tcp_server_shutdown_receiver) = oneshot::channel();

    let endpoint_contacts = EndpointContactList::new();

    let endpoint_worker = EndpointWorker::new(
        command_receiver,
        event_sender,
        internal_event_receiver,
        internal_event_sender.clone(),
        endpoint_contacts.clone(),
        endpoint_worker_shutdown_receiver,
    );

    let binding_address = config.socket_address();
    let tcp_server = TcpServer::new(
        binding_address,
        internal_event_sender,
        tcp_server_shutdown_receiver,
        endpoint_contacts,
    )
    .await;

    let endpoint_worker = tokio::spawn(endpoint_worker.run());
    let tcp_server = tokio::spawn(tcp_server.run());

    shutdown.add_worker_shutdown(endpoint_worker_shutdown_sender, endpoint_worker);
    shutdown.add_worker_shutdown(tcp_server_shutdown_sender, tcp_server);

    MAX_TCP_BUFFER_SIZE.swap(config.max_tcp_buffer_size, Ordering::Relaxed);
    RECONNECT_INTERVAL.swap(config.reconnect_interval, Ordering::Relaxed);

    (Network::new(config, command_sender), event_receiver)
}

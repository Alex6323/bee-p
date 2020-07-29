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

//! # bee-network
//!
//! Network layer for Bee.
//!
//! The main properties of its architecure are:
//! * async (depends on async_std),
//! * message passing in favor of shared state (mpsc, oneshot channels),
//! * event-driven (situations are modeled as events),
//! * command pattern
//! * no unsafe code
//! * very few dependencies
//! * well documented

#![warn(missing_docs)]

pub use address::{url::Url, Address, Port};
pub use commands::{response_channel, Command, Requester, Responder};
pub use config::{NetworkConfig, NetworkConfigBuilder};
pub use endpoint::{origin::Origin, Endpoint, EndpointId};
pub use events::{Event, EventSubscriber};

pub use network::Network;

mod address;
mod commands;
mod constants;
mod endpoint;
mod events;
mod network;
mod tcp;
// mod udp;
mod config;
mod utils;

use endpoint::{allowlist, worker::EndpointWorker as EpWorker};
use events::EventSubscriber as Events;
use tcp::worker::TcpWorker;
// use udp::worker::UdpWorker;

use bee_common::shutdown::Shutdown;

use async_std::task::spawn;
use futures::channel::oneshot;

/// Initializes the network layer.
pub fn init(config: NetworkConfig, shutdown: &mut Shutdown) -> (Network, Events) {
    let (command_sender, commands) = commands::command_channel();

    let (event_sender, events) = events::event_channel();

    let (internal_event_sender, internal_events) = events::event_channel();

    let (epw_sd_sender, epw_shutdown) = oneshot::channel();

    let (tcp_sd_sender, tcp_shutdown) = oneshot::channel();
    // let (udp_sd_sender, udp_shutdown) = oneshot::channel();

    let ep_worker = EpWorker::new(
        commands,
        internal_events,
        internal_event_sender.clone(),
        event_sender,
        config.reconnect_interval,
    );

    let tcp_worker = TcpWorker::new(config.socket_addr(), internal_event_sender);
    // let udp_worker = UdpWorker::new(binding_addr, internal_event_sender.clone(), udp_shutdown);

    shutdown.add_worker_shutdown(epw_sd_sender, spawn(ep_worker.run(epw_shutdown)));
    shutdown.add_worker_shutdown(tcp_sd_sender, spawn(tcp_worker.run(tcp_shutdown)));
    // shutdown.add_worker_shutdown(udp_sd_sender, spawn(udp_worker.run()));

    allowlist::init();
    shutdown.add_action(|| allowlist::drop());

    (Network::new(config, command_sender), events)
}

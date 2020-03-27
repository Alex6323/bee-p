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

#![deny(missing_docs)]
#![recursion_limit = "1024"]

pub use address::{
    url::Url,
    Address,
};
pub use commands::Command;
pub use commands::{
    response_channel,
    Requester,
    Responder,
};
pub use endpoint::{
    role::Role,
    Endpoint,
    EndpointId,
};
pub use events::{
    Event,
    EventSubscriber,
};
pub use network::Network;
pub use shutdown::Shutdown;

mod address;
mod commands;
mod constants;
mod endpoint;
mod errors;
mod events;
mod network;
mod shutdown;
mod tcp;
mod udp;
mod utils;

use endpoint::worker::EndpointWorker as EpWorker;
use events::EventSubscriber as Events;
use tcp::worker::TcpWorker;
//use udp::worker::UdpWorker;

use async_std::task::spawn;
use futures::channel::oneshot;

/// Initializes the network layer.
pub fn init(binding_addr: Address) -> (Network, Shutdown, Events) {
    let (command_sender, commands) = commands::command_channel();
    let (event_sender, events) = events::event_channel();
    let (internal_event_sender, internal_events) = events::event_channel();

    let mut shutdown = Shutdown::new();

    let (epw_sd_sender, epw_shutdown) = oneshot::channel();
    let (tcp_sd_sender, tcp_shutdown) = oneshot::channel();
    //let (udp_sd_sender, udp_shutdown) = oneshot::channel();

    let ep_worker = EpWorker::new(
        commands,
        internal_events,
        epw_shutdown,
        internal_event_sender.clone(),
        event_sender.clone(),
    );

    let tcp_worker = TcpWorker::new(binding_addr, internal_event_sender.clone(), tcp_shutdown);
    //let udp_worker = UdpWorker::new(binding_addr, internal_event_sender.clone(), udp_shutdown);

    shutdown.add_notifier(epw_sd_sender);
    shutdown.add_notifier(tcp_sd_sender);
    //shutdown.add_notifier(udp_sd_sender);

    shutdown.add_task(spawn(ep_worker.run()));
    shutdown.add_task(spawn(tcp_worker.run()));
    //shutdown.add_task(spawn(udp_worker.run()));

    let network = Network::new(command_sender);

    (network, shutdown, events)
}

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

use endpoint::actor::EndpointActor as EdpActor;
use events::EventSubscriber as Events;
use tcp::actor::TcpActor;
//use udp::actor::UdpActor;

use async_std::task::spawn;
use futures::channel::oneshot;

/// Initializes the network layer.
pub fn init(binding_addr: Address) -> (Network, Shutdown, Events) {
    let (command_sender, commands) = commands::command_channel();
    let (event_sender, events) = events::event_channel();
    let (internal_event_sender, internal_events) = events::event_channel();

    let mut shutdown = Shutdown::new();

    let (edp_sd_sender, edp_shutdown) = oneshot::channel();
    let (tcp_sd_sender, tcp_shutdown) = oneshot::channel();
    //let (udp_sd_sender, udp_shutdown) = oneshot::channel();

    let edp_actor = EdpActor::new(
        commands,
        internal_events,
        edp_shutdown,
        internal_event_sender.clone(),
        event_sender.clone(),
    );

    let tcp_actor = TcpActor::new(binding_addr, internal_event_sender.clone(), tcp_shutdown);
    //let udp_actor = UdpActor::new(binding_addr, internal_event_sender, udp_shutdown);

    shutdown.add_notifier(edp_sd_sender);
    shutdown.add_notifier(tcp_sd_sender);
    //shutdown.add_notifier(udp_sd_sender);

    shutdown.add_task(spawn(edp_actor.run()));
    shutdown.add_task(spawn(tcp_actor.run()));
    //shutdown.add_task(spawn(udp_actor.run()));

    let network = Network::new(command_sender);

    (network, shutdown, events)
}

mod constants;
mod logger;
mod node;

use constants::{
    BEE_NAME,
    BEE_VERSION,
};
use node::Node;

use bee_network::Address;

use async_std::task::block_on;
use log::info;

fn main() {
    // TODO conf variable
    logger::init(log::LevelFilter::Debug);

    info!("[Main ] Welcome to {} {}!", BEE_NAME, BEE_VERSION);

    let addr = block_on(Address::from_host_addr("localhost:1337")).unwrap();
    let (network, shutdown, receiver) = bee_network::init(addr);

    let mut node = Node::new(network, shutdown, receiver);

    block_on(node.init());
    block_on(node.run());
}

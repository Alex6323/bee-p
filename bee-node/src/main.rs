mod conf;
mod constants;
mod node;

use conf::{
    NodeConfBuilder,
    CONF_PATH,
};
use node::Node;

use bee_network::Address;

use async_std::task::block_on;
use std::fs;

fn main() {
    // TODO handle error
    let conf_builder = match fs::read_to_string(CONF_PATH) {
        Ok(toml_str) => match toml::from_str::<NodeConfBuilder>(&toml_str) {
            Ok(conf_builder) => conf_builder,
            Err(_) => return,
        },
        Err(_) => return,
    };

    let conf = conf_builder.build();

    let addr = block_on(Address::from_addr_str("localhost:1337")).unwrap();
    let (network, shutdown, receiver) = bee_network::init(conf.network.clone(), addr);

    let mut node = Node::new(conf, network, shutdown, receiver);

    block_on(node.init());
    block_on(node.run());
}

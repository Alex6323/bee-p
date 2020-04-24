mod conf;
mod constants;
mod node;

use conf::{
    NodeConfBuilder,
    CONF_PATH,
};

use node::Node;

use async_std::task::block_on;

use std::fs;

fn main() {
    let conf_builder = match fs::read_to_string(CONF_PATH) {
        Ok(toml_str) => match toml::from_str::<NodeConfBuilder>(&toml_str) {
            Ok(conf_builder) => conf_builder,
            Err(e) => {
                panic!("[Node ] Error parsing .toml config file.\n{:?}", e);
            }
        },
        Err(e) => {
            panic!("[Node ] Error reading .toml config file.\n{:?}", e);
        }
    };

    let conf = conf_builder.build();

    println!("{:?}", conf.network);

    let (network, shutdown, receiver) = bee_network::init(conf.network.clone());

    let mut node = Node::new(conf, network, shutdown, receiver);

    block_on(node.init());
    block_on(node.run());
}

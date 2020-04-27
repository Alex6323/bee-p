mod config;
mod constants;
mod node;

use config::{
    NodeConfigBuilder,
    CONFIG_PATH,
};

use node::Node;

use async_std::task::block_on;

use std::fs;

fn main() {
    let config_builder = match fs::read_to_string(CONFIG_PATH) {
        Ok(toml) => match toml::from_str::<NodeConfigBuilder>(&toml) {
            Ok(config_builder) => config_builder,
            Err(e) => {
                panic!("[Node ] Error parsing .toml config file.\n{:?}", e);
            }
        },
        Err(e) => {
            panic!("[Node ] Error reading .toml config file.\n{:?}", e);
        }
    };

    let config = config_builder.build();

    let (network, shutdown, receiver) = bee_network::init(config.network);

    //TODO: proper shutdown
    let mut node = Node::new(config, network, shutdown, receiver);

    block_on(node.init());
    block_on(node.run());
}

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

mod cli;
mod config;
mod constants;
mod node;

use cli::CliArgs;
use config::{NodeConfigBuilder, CONFIG_PATH};
use node::Node;

use async_std::task::block_on;

use std::fs;

fn main() {
    let mut config_builder = match fs::read_to_string(CONFIG_PATH) {
        Ok(toml) => match toml::from_str::<NodeConfigBuilder>(&toml) {
            Ok(config_builder) => config_builder,
            Err(e) => {
                panic!("[Node ] Error parsing config file: {:?}", e);
            }
        },
        Err(e) => {
            panic!("[Node ] Error reading config file: {:?}", e);
        }
    };

    CliArgs::new().apply_to_config(&mut config_builder);

    let config = config_builder.finish();

    let (network, shutdown, receiver) = bee_network::init(config.network);

    // TODO: proper shutdown
    let mut node = Node::new(config, network, shutdown, receiver);

    block_on(node.init());
    block_on(node.run());
}

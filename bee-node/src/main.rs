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

use bee_common::logger::logger_init;
use bee_common_ext::node::{Node as _, NodeBuilder as _};
use bee_node::{default_plugins, CliArgs, Node, NodeConfigBuilder};
use bee_storage_rocksdb::storage::Storage as Rocksdb;

use log::error;

const CONFIG_PATH: &str = "./config.toml";

#[tokio::main]
async fn main() {
    let config = NodeConfigBuilder::from_file(CONFIG_PATH)
        .expect("Error when creating node config builder")
        .with_cli_args(CliArgs::default())
        .finish();

    Node::<Rocksdb>::build(config)
        .with_plugin::<default_plugins::Mps>()
        .with_logging()
        .finish()
        .await
        .expect("Failed to build node")
        .run()
        .await
        .expect("Failed to run node");
}

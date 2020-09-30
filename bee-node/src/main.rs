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
use bee_node::{CliArgs, Node, NodeConfigBuilder};

const CONFIG_PATH: &str = "./config.toml";

#[tokio::main]
async fn main() {
    match NodeConfigBuilder::from_file(CONFIG_PATH) {
        Ok(mut config_builder) => {
            CliArgs::default().apply_to_config(&mut config_builder);
            let config = config_builder.finish();

            logger_init(config.logger.clone()).unwrap();

            match Node::<bee_storage_rocksdb::storage::Storage>::builder(config).finish().await {
                Ok(node) => {
                    if let Err(e) = node.run().await {
                        eprintln!("Program aborted. Error was: {}", e);
                    }
                }
                Err(e) => {
                    eprintln!("Program aborted. Error was: {}", e);
                }
            }
        }
        Err(e) => {
            eprintln!("Program aborted. Error was: {}", e);
        }
    }
}

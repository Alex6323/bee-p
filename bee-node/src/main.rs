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

use bee_common::logger_init;
use bee_node::{read_config, BeeNode, CliArgs};

fn main() {
    match read_config() {
        Ok(mut config) => {
            CliArgs::new().apply_to_config(&mut config);
            let config = config.finish();

            logger_init(config.logger.clone()).unwrap();

            match BeeNode::build(config).finish() {
                Ok(mut bee) => {
                    bee.run_loop();
                    bee.shutdown();
                }
                Err(_) => {
                    // TODO use error
                    eprintln!("Unable to create a Bee node. Program aborted.");
                }
            }
        }
        Err(_) => {
            // TODO use error
            eprintln!("Unable to read local config file. Program aborted.");
        }
    }
}

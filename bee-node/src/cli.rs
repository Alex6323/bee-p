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

use crate::config::NodeConfigBuilder;

use bee_common::logger::LOGGER_STDOUT_NAME;

use log::LevelFilter;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct CliArgs {
    #[structopt(
        short = "l",
        long = "log-level",
        help = "Stdout log level amongst \"trace\", \"debug\", \"info\", \"warn\" and \"error\""
    )]
    log_level: Option<LevelFilter>,
}

impl CliArgs {
    pub fn new() -> Self {
        Self::from_args()
    }

    pub fn apply_to_config(self, config: &mut NodeConfigBuilder) {
        self.log_level
            .map(|log_level| config.logger.level(LOGGER_STDOUT_NAME, log_level));
    }
}

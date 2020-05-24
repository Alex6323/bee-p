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

use log::LevelFilter;
use serde::Deserialize;

const DEFAULT_COLOR: bool = true;
const DEFAULT_FILES: Vec<String> = Vec::new();
const DEFAULT_LEVEL: &str = "info";
const DEFAULT_STDOUT: bool = true;

#[derive(Default, Deserialize)]
pub struct LoggerConfigBuilder {
    color: Option<bool>,
    files: Option<Vec<String>>,
    level: Option<String>,
    stdout: Option<bool>,
}

impl LoggerConfigBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn color(mut self, color: bool) -> Self {
        self.color.replace(color);
        self
    }

    pub fn add_file(mut self, file: &str) {
        if self.files.is_none() {
            self.files.replace(Vec::new());
        }
        self.files.unwrap().push(file.to_owned());
    }

    pub fn log_level(mut self, log_level: &str) -> Self {
        self.level.replace(log_level.to_string());
        self
    }

    pub fn stdout(mut self, stdout: bool) -> Self {
        self.stdout.replace(stdout);
        self
    }

    pub fn finish(self) -> LoggerConfig {
        let level = match self.level.unwrap_or_else(|| DEFAULT_LEVEL.to_owned()).as_str() {
            "trace" => LevelFilter::Trace,
            "debug" => LevelFilter::Debug,
            "info" => LevelFilter::Info,
            "warn" => LevelFilter::Warn,
            "error" => LevelFilter::Error,
            _ => LevelFilter::Info,
        };

        LoggerConfig {
            color: self.color.unwrap_or(DEFAULT_COLOR),
            files: self.files.unwrap_or(DEFAULT_FILES),
            level,
            stdout: self.stdout.unwrap_or(DEFAULT_STDOUT),
        }
    }
}

#[derive(Clone)]
pub struct LoggerConfig {
    pub(crate) color: bool,
    pub(crate) files: Vec<String>,
    pub(crate) level: LevelFilter,
    pub(crate) stdout: bool,
}

impl LoggerConfig {
    pub fn build() -> LoggerConfigBuilder {
        LoggerConfigBuilder::new()
    }
}

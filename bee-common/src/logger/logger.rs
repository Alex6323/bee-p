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

// TODO: beautify datetime output
// TODO: log to files
// TODO: handle stdout true/false

use crate::LoggerConfig;

use std::io::Write;

use env_logger::fmt::Color;
use log::Level;

pub fn init(config: LoggerConfig) {
    let conf = config.clone();

    pretty_env_logger::formatted_timed_builder()
        .format_indent(None)
        .format(move |f, record| {
            let ts = f.timestamp();

            let mut level_style = f.style();

            if conf.color {
                let color = match record.level() {
                    Level::Trace => Color::Magenta,
                    Level::Debug => Color::Blue,
                    Level::Info => Color::Green,
                    Level::Warn => Color::Yellow,
                    Level::Error => Color::Red,
                };
                level_style.set_color(color).set_bold(true);
            }

            writeln!(
                f,
                "[{}][{:>5}][{}] {}",
                ts,
                level_style.value(record.level()),
                record.target(),
                record.args()
            )
        })
        .format_timestamp_secs()
        .filter_level(config.level)
        .init();
}

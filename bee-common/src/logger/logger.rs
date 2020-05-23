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

use env_logger::fmt::Color;

use std::io::Write;

pub fn init(level_filter: log::LevelFilter) {
    pretty_env_logger::formatted_timed_builder()
        .format_indent(None)
        .format(|f, record| {
            let ts = f.timestamp();

            let col = match record.level() {
                log::Level::Trace => Color::Magenta,
                log::Level::Debug => Color::Blue,
                log::Level::Info => Color::Green,
                log::Level::Warn => Color::Yellow,
                log::Level::Error => Color::Red,
            };

            let mut level_style = f.style();
            level_style.set_color(col).set_bold(true);

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
        .filter_level(level_filter)
        .init();
}

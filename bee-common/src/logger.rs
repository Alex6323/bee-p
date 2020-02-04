use log::{log_enabled, trace, debug, info, warn, error};
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

            writeln!(f, "[{} {:>7}] {}", ts, level_style.value(record.level()), record.args())
        })
        //.format_timestamp(Some(env_logger::TimestampPrecision::Millis))
        .format_timestamp_secs()
        .filter_level(level_filter)
        .init();
}

pub fn trace(message: &str) {
    if log_enabled!(log::Level::Trace) {
        trace!("{}", message);
    }
}

pub fn debug(message: &str) {
    if log_enabled!(log::Level::Debug) {
        debug!("{}", message);
    }
}

pub fn info(message: &str) {
    if log_enabled!(log::Level::Info) {
        info!("{}", message);
    }
}

pub fn warn(message: &str) {
    if log_enabled!(log::Level::Warn) {
        warn!("{}", message);
    }
}

pub fn error(message: &str) {
    if log_enabled!(log::Level::Error) {
        error!("{}", message);
    }
}
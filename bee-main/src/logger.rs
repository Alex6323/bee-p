use common::constants::BEE_LOG;

use log::*;
use std::io::Write;

use env_logger::fmt::Color;

pub struct LogLevel(Level);

impl LogLevel {
    pub fn color(&self) -> Color {
        match *&self.0 {
            Level::Trace => Color::Magenta,
            Level::Debug => Color::Blue,
            Level::Info => Color::Green,
            Level::Warn => Color::Yellow,
            Level::Error => Color::Red,
        }
    }
}

impl std::fmt::Display for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *&self.0 {
            Level::Trace => write!(f, "{}", "trace"),
            Level::Debug => write!(f, "{}", "debug"),
            Level::Info => write!(f, "{}", "info"),
            Level::Warn => write!(f, "{}", "warn"),
            Level::Error => write!(f, "{}", "error"),
        }
    }
}

pub struct BeeLogger {
    log_level: LogLevel,
 }

impl BeeLogger {
    pub fn new(log_level: Level) -> Self {
        Self { log_level: LogLevel(log_level) }
    }

    pub fn init(&self) {
        //std::env::set_var(BEE_LOG, self.log_level.to_string());
        //pretty_env_logger::init_custom_env(BEE_LOG);
        //env_logger::
        pretty_env_logger::formatted_timed_builder()
            .format_indent(None)
            .format(|f, record| {
                let ts = f.timestamp();

                let col = match record.level() {
                    Level::Trace => Color::Magenta,
                    Level::Debug => Color::Blue,
                    Level::Info => Color::Green,
                    Level::Warn => Color::Yellow,
                    Level::Error => Color::Red,
                };

                let mut level_style = f.style();
                level_style.set_color(col).set_bold(true);

                writeln!(f, "[{} {:>7}] {}", ts, level_style.value(record.level()), record.args())
            })
            //.format_timestamp(Some(env_logger::TimestampPrecision::Millis))
            .format_timestamp_secs()
            .filter_level(LevelFilter::Trace)
            .init();


        //let mut builder = pretty_env_logger::formatted_timed_builder();
        //let logger = builder.build();
    }

    pub fn exit(&self) {
        //std::env::remove_var(BEE_LOG);
    }

    pub fn trace(&self, message: &str) {
        if log_enabled!(Level::Trace) {
            trace!("{}", message);
        }
    }

    pub fn debug(&self, message: &str) {
        if log_enabled!(Level::Debug) {
            debug!("{}", message);
        }
    }

    pub fn info(&self, message: &str) {
        if log_enabled!(Level::Info) {
            info!("{}", message);
        }
    }

    pub fn warn(&self, message: &str) {
        if log_enabled!(Level::Warn) {
            warn!("{}", message);
        }
    }

    pub fn error(&self, message: &str) {
        if log_enabled!(Level::Error) {
            error!("{}", message);
        }
    }
}
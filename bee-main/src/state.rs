use std::fmt;

use common::constants::BEE_DISPLAYED_NAME;

#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum State {
    BootingUp,
    Running,
    ShuttingDown,
}

impl fmt::Display for State {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            State::BootingUp => write!(f, "{} is booting up.", BEE_DISPLAYED_NAME),
            State::Running => write!(f, "{} is running.", BEE_DISPLAYED_NAME),
            State::ShuttingDown => write!(f, "{} is shutting down.", BEE_DISPLAYED_NAME),
        }
    }
}
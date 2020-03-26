use std::fmt;

/// The connection relationship with an endpoint.
#[derive(Clone, Debug)]
pub enum Role {
    /// Outgoing connection attempt that got accepted.
    Client,

    /// Incoming connection attempt that got accepted.
    Server,

    /// Participating endpoints don't have a specified role assigned.
    Unspecified,
}

impl fmt::Display for Role {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Role::Client => write!(f, "Client"),
            Role::Server => write!(f, "Server"),
            Role::Unspecified => write!(f, "Unspecified"),
        }
    }
}

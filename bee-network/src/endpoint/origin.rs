use std::fmt;

/// The connection relationship with an endpoint.
#[derive(Clone, Debug)]
pub enum Origin {
    /// Incoming connection attempt that got accepted.
    Inbound,

    /// Outgoing connection attempt that got accepted.
    Outbound,

    /// Participating endpoints are not bound to eachother.
    Unbound,
}

impl fmt::Display for Origin {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Origin::Outbound => write!(f, "outbound"),
            Origin::Inbound => write!(f, "inbound"),
            Origin::Unbound => write!(f, "unbound"),
        }
    }
}

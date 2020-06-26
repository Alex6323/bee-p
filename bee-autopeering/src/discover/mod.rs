pub(crate) mod worker;

use std::time::Duration;

const HOUR: u64 = 3600; // in seconds

/// The time until a peeer verification expires.
pub const PING_EXPIRATION: Duration = Duration::from_secs(12 * HOUR);

/// Maximum number of peers returned in `DiscoveryResponse`.
pub const MAX_PEERS_IN_RESPONSE: usize = 6;

/// Maximum number of services a peer can support.
pub const MAX_SERVICES: usize = 5;

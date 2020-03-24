pub(crate) const COORDINATOR: &str =
    "EQSAUZXULTTYZCLNJNTXQTQHOMOFZERHTCGTXOLTVAHKSA9OGAZDEKECURBRIXIJWNPFCQIOVFVVXJVD9";
pub(crate) const COORDINATOR_BYTES: [u8; 49] = [
    234, 56, 202, 174, 238, 197, 195, 253, 109, 14, 137, 227, 44, 144, 151, 188, 192, 45, 220, 236, 64, 168, 220, 197,
    22, 199, 188, 1, 45, 11, 107, 190, 49, 84, 147, 176, 184, 108, 223, 189, 17, 167, 184, 240, 213, 170, 111, 34, 0,
];
pub(crate) const COORDINATOR_SECURITY_LEVEL: u8 = 2;
pub(crate) const COORDINATOR_DEPTH: u8 = 23;
pub(crate) const MINIMUM_WEIGHT_MAGNITUDE: u8 = 14;

// TODO should be pub(crate) and just serve as default for conf
pub const HANDSHAKE_SEND_BOUND: usize = 1000;
pub const MILESTONE_REQUEST_SEND_BOUND: usize = 1000;
pub const TRANSACTION_BROADCAST_SEND_BOUND: usize = 1000;
pub const TRANSACTION_REQUEST_SEND_BOUND: usize = 1000;
pub const HEARTBEAT_SEND_BOUND: usize = 1000;

// TODO move out of here
pub(crate) fn slice_eq(a: &[u8; 49], b: &[u8; 49]) -> bool {
    if a.len() != b.len() {
        return false;
    }

    for i in 0..a.len() {
        if a[i] != b[i] {
            return false;
        }
    }

    true
}

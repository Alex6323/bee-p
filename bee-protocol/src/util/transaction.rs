use bee_common::constants::TRANSACTION_BYTE_LEN;

// TODO constants
pub(crate) fn uncompress_bytes(bytes: &[u8]) -> [u8; TRANSACTION_BYTE_LEN] {
    let mut uncompressed_bytes = [0u8; TRANSACTION_BYTE_LEN];
    let payload_size = bytes.len() - 292;

    uncompressed_bytes[..payload_size].copy_from_slice(&bytes[..payload_size]);
    uncompressed_bytes[1312..].copy_from_slice(&bytes[payload_size..]);

    uncompressed_bytes
}

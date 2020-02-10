pub trait Message {
    fn from_bytes(bytes: &[u8]) -> Self;

    fn to_bytes() -> Vec<u8>;

    fn size() -> (usize, usize);
}

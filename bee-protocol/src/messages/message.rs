pub trait Message {
    fn size() -> (usize, usize);

    fn from_bytes(bytes: &[u8]) -> Self;

    fn to_bytes(self) -> Vec<u8>;
}

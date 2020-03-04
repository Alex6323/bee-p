pub enum NeighborEvent {
    Connected,
    Disconnected,
    Message { size: usize, bytes: Vec<u8> },
}

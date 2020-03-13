pub enum NeighborEvent {
    Removed,
    Connected,
    Disconnected,
    Message { size: usize, bytes: Vec<u8> },
}

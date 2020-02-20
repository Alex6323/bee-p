use crate::message::Handshake;
use crate::neighbor::Neighbor;
use crate::processing::{ProcessingState, ProcessingUnit};

struct HandshakeState;

impl ProcessingState for HandshakeState {}

impl<'a> ProcessingUnit<'a, Handshake, HandshakeState> {
    pub fn new(message: Handshake, neighbor: &'a Neighbor) -> Self {
        Self {
            message: Box::new(message),
            neighbor: neighbor,
            state: HandshakeState {},
        }
    }

    pub fn process(self) {}
}

type HandshakeProcessor<'a> = ProcessingUnit<'a, Handshake, HandshakeState>;

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn new_processor_test() {
        let port = 0xcd98;
        let timestamp = 0xb2a1d7546a470ed8;
        let coordinator = [
            160, 3, 36, 228, 202, 18, 56, 37, 229, 28, 240, 65, 225, 238, 64, 55, 244, 83, 155,
            232, 31, 255, 208, 9, 126, 21, 82, 57, 180, 237, 182, 101, 242, 57, 202, 28, 118, 203,
            67, 93, 74, 238, 57, 39, 51, 169, 193, 124, 254,
        ];
        let minimum_weight_magnitude = 0x6e;
        let supported_messages = [
            0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0,
        ];
        let message = Handshake::new(
            port,
            timestamp,
            &coordinator,
            minimum_weight_magnitude,
            &supported_messages,
        );
        let neighbor = Neighbor::new();
        let processor = HandshakeProcessor::new(message, &neighbor).process();
    }
}

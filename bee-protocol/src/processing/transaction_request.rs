use crate::message::TransactionRequest;
use crate::neighbor::Neighbor;
use crate::processing::{ProcessingState, ProcessingUnit};

struct TransactionRequestState;

impl ProcessingState for TransactionRequestState {}

impl<'a> ProcessingUnit<'a, TransactionRequest, TransactionRequestState> {
    pub fn new(message: TransactionRequest, neighbor: &'a Neighbor) -> Self {
        Self {
            message: Box::new(message),
            neighbor: neighbor,
            state: TransactionRequestState {},
        }
    }
}

type TransactionRequestProcessor<'a> =
    ProcessingUnit<'a, TransactionRequest, TransactionRequestState>;

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn new_processor_test() {
        let hash = [
            160, 3, 36, 228, 202, 18, 56, 37, 229, 28, 240, 65, 225, 238, 64, 55, 244, 83, 155,
            232, 31, 255, 208, 9, 126, 21, 82, 57, 180, 237, 182, 101, 242, 57, 202, 28, 118, 203,
            67, 93, 74, 238, 57, 39, 51, 169, 193, 124, 254,
        ];
        let message = TransactionRequest::new(hash);
        let neighbor = Neighbor::new();
        let _processor = TransactionRequestProcessor::new(message, &neighbor);
    }
}

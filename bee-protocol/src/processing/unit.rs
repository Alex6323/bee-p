use crate::neighbor::Neighbor;
use bee_network::Message;

pub(crate) trait ProcessingState {}

pub(crate) struct ProcessingUnit<'a, M: Message, S: ProcessingState> {
    pub(crate) message: Box<M>,
    pub(crate) neighbor: &'a Neighbor,
    pub(crate) state: S,
}

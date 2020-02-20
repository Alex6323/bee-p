use crate::message::Message;
use crate::neighbor::Neighbor;

pub(crate) trait ProcessingState {}

pub(crate) struct ProcessingUnit<'a, M: Message, S: ProcessingState> {
    pub(crate) message: Box<M>,
    pub(crate) neighbor: &'a Neighbor,
    pub(crate) state: S,
}

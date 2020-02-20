use bee_network::Message;

pub(crate) trait ProcessingState {}

pub(crate) struct ProcessingUnit<M: Message, S: ProcessingState> {
    pub(crate) message: Box<M>,
    pub(crate) state: S,
}

use bee_ternary::{
    T1B1Buf,
    TritBuf,
};

use futures::{
    channel::mpsc::Receiver,
    stream::StreamExt,
};
use log::info;

pub(crate) type MilestoneValidatorWorkerEvent = TritBuf<T1B1Buf>;

pub(crate) struct MilestoneValidatorWorker {
    receiver: Receiver<MilestoneValidatorWorkerEvent>,
}

impl MilestoneValidatorWorker {
    pub(crate) fn new(receiver: Receiver<MilestoneValidatorWorkerEvent>) -> Self {
        Self { receiver: receiver }
    }

    pub(crate) async fn run(mut self) {
        info!("[MilestoneValidatorWorker ] Running.");

        while let Some(_hash) = self.receiver.next().await {}
    }
}

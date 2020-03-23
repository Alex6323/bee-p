use bee_ternary::{
    T1B1Buf,
    TritBuf,
};

use futures::channel::mpsc::Receiver;
use futures::stream::StreamExt;
use log::info;

pub enum MilestoneValidatorWorkerEvent {
    Candidate(TritBuf<T1B1Buf>),
}

pub struct MilestoneValidatorWorker {
    receiver: Receiver<MilestoneValidatorWorkerEvent>,
}

impl MilestoneValidatorWorker {
    pub fn new(receiver: Receiver<MilestoneValidatorWorkerEvent>) -> Self {
        Self { receiver: receiver }
    }

    pub async fn run(mut self) {
        info!("[MilestoneValidatorWorker ] Running.");

        while let Some(event) = self.receiver.next().await {}
    }
}

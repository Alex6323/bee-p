use crate::{
    message::MilestoneRequest,
    milestone::MilestoneIndex,
    protocol::Protocol,
    worker::SenderWorker,
};

use bee_network::EndpointId;
use bee_tangle::tangle;

use std::cmp::Ordering;

use futures::{
    channel::oneshot,
    future::FutureExt,
    select,
};
use log::info;

#[derive(Eq, PartialEq)]
pub(crate) struct MilestoneRequesterWorkerEntry(pub(crate) MilestoneIndex, pub(crate) Option<EndpointId>);

// TODO check that this is the right order
impl PartialOrd for MilestoneRequesterWorkerEntry {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

impl Ord for MilestoneRequesterWorkerEntry {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.cmp(&other.0)
    }
}

pub(crate) struct MilestoneRequesterWorker {}

impl MilestoneRequesterWorker {
    pub(crate) fn new() -> Self {
        Self {}
    }

    pub(crate) async fn run(self, shutdown: oneshot::Receiver<()>) {
        info!("[MilestoneRequesterWorker ] Running.");

        let mut shutdown_fused = shutdown.fuse();

        loop {
            select! {
                // TODO impl fused stream
                entry = Protocol::get().milestone_requester_worker.0.pop().fuse() => {
                    if let MilestoneRequesterWorkerEntry(index, _opt_epid) = entry {
                        if tangle().contains_milestone(&(index.into())) {
                            continue;
                        }
                        // let epid = match opt_epid {
                        //     Some(epid) => epid,
                        //     // TODO random epid ?
                        //     None => ()
                        // };
                        // SenderWorker::<MilestoneRequest>::send(&epid, MilestoneRequest::new(index)).await;
                    }
                },
                _ = shutdown_fused => {
                    break;
                }
            }
        }

        info!("[MilestoneRequesterWorker ] Stopped.");
    }
}

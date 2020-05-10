use crate::{message::MilestoneRequest, milestone::MilestoneIndex, protocol::Protocol, worker::SenderWorker};

use bee_network::EndpointId;
use bee_tangle::tangle;

use std::cmp::Ordering;

use futures::{channel::oneshot, future::FutureExt, select};
use log::info;
use rand::{Rng, SeedableRng};
use rand_pcg::Pcg32;

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

pub(crate) struct MilestoneRequesterWorker {
    rng: Pcg32,
}

impl MilestoneRequesterWorker {
    pub(crate) fn new() -> Self {
        Self {
            rng: Pcg32::from_entropy(),
        }
    }

    async fn process_request(&mut self, index: MilestoneIndex, epid: Option<EndpointId>) {
        if Protocol::get().contexts.is_empty() {
            return;
        }

        // TODO check that it has the milestone
        let epid = match epid {
            Some(epid) => epid,
            None => {
                match Protocol::get()
                    .contexts
                    .iter()
                    .nth(self.rng.gen_range(0, Protocol::get().contexts.len()))
                {
                    Some(entry) => *entry.key(),
                    None => return,
                }
            }
        };

        SenderWorker::<MilestoneRequest>::send(&epid, MilestoneRequest::new(index)).await;
    }

    pub(crate) async fn run(mut self, shutdown: oneshot::Receiver<()>) {
        info!("[MilestoneRequesterWorker ] Running.");

        let mut shutdown_fused = shutdown.fuse();

        loop {
            select! {
                // TODO impl fused stream
                entry = Protocol::get().milestone_requester_worker.0.pop().fuse() => {
                    if let MilestoneRequesterWorkerEntry(index, epid) = entry {
                        if !tangle().contains_milestone(index.into()) {
                            self.process_request(index, epid).await;
                        }

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

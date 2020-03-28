use crate::{
    message::MilestoneRequest,
    milestone::MilestoneIndex,
    worker::SenderWorker,
};

use bee_network::EndpointId;

pub async fn send_milestone_request(epid: EndpointId, index: MilestoneIndex) {
    SenderWorker::<MilestoneRequest>::send(&epid, MilestoneRequest::new(index)).await;
}

pub async fn broadcast_milestone_request(index: MilestoneIndex) {
    SenderWorker::<MilestoneRequest>::broadcast(MilestoneRequest::new(index)).await;
}

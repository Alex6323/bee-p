use crate::{
    message::Heartbeat,
    milestone::MilestoneIndex,
    worker::SenderWorker,
};

use bee_network::EndpointId;

pub async fn send_heartbeat(
    epid: EndpointId,
    first_solid_milestone_index: MilestoneIndex,
    last_solid_milestone_index: MilestoneIndex,
) {
    SenderWorker::<Heartbeat>::send(
        &epid,
        Heartbeat::new(first_solid_milestone_index, last_solid_milestone_index),
    )
    .await;
}

pub async fn broadcast_heartbeat(
    first_solid_milestone_index: MilestoneIndex,
    last_solid_milestone_index: MilestoneIndex,
) {
    SenderWorker::<Heartbeat>::broadcast(Heartbeat::new(first_solid_milestone_index, last_solid_milestone_index)).await;
}

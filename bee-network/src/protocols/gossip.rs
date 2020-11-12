use futures::prelude::*;
use libp2p::core::{
    muxing::{StreamMuxerBox, SubstreamRef},
    InboundUpgrade, Negotiated, OutboundUpgrade, UpgradeInfo,
};

use std::{iter, sync::Arc};

const PROTOCOL_INFO: &[u8] = b"/iota-gossip/1/1.0.0";

pub type GossipSubstream = Negotiated<SubstreamRef<Arc<StreamMuxerBox>>>;

#[derive(Default, Debug, Copy, Clone)]
pub struct GossipProtocol;

impl UpgradeInfo for GossipProtocol {
    type Info = &'static [u8];
    type InfoIter = iter::Once<Self::Info>;

    fn protocol_info(&self) -> Self::InfoIter {
        iter::once(PROTOCOL_INFO)
    }
}

impl InboundUpgrade<GossipSubstream> for GossipProtocol {
    type Output = GossipSubstream;
    type Error = ();
    type Future = future::Ready<Result<Self::Output, Self::Error>>;

    fn upgrade_inbound(self, stream: GossipSubstream, _: Self::Info) -> Self::Future {
        future::ok(stream)
    }
}

impl OutboundUpgrade<GossipSubstream> for GossipProtocol {
    type Output = GossipSubstream;
    type Error = ();
    type Future = future::Ready<Result<Self::Output, Self::Error>>;

    fn upgrade_outbound(self, stream: GossipSubstream, _: Self::Info) -> Self::Future {
        future::ok(stream)
    }
}

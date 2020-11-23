// Copyright 2020 IOTA Stiftung
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file except in compliance with
// the License. You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software distributed under the License is distributed on
// an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and limitations under the License.

use crate::NETWORK_ID;

use futures::prelude::*;
use libp2p::core::{
    muxing::{StreamMuxerBox, SubstreamRef},
    InboundUpgrade, Negotiated, OutboundUpgrade, UpgradeInfo,
};

use std::{
    iter,
    sync::{atomic::Ordering, Arc},
};

pub type GossipSubstream = Negotiated<SubstreamRef<Arc<StreamMuxerBox>>>;

#[derive(Default, Debug, Clone)]
pub struct GossipProtocol;

impl UpgradeInfo for GossipProtocol {
    type Info = Vec<u8>;
    type InfoIter = iter::Once<Self::Info>;

    fn protocol_info(&self) -> Self::InfoIter {
        iter::once(
            format!("/iota-gossip/{}/1.0.0", NETWORK_ID.load(Ordering::Relaxed))
                .as_bytes()
                .to_vec(),
        )
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

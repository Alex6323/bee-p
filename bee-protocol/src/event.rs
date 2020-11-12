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

use crate::Milestone;

use bee_message::MessageId;
use bee_network::{Multiaddr, PeerId};

pub struct HandshakeCompleted(pub PeerId, pub Multiaddr);

pub struct LatestMilestoneChanged(pub Milestone);

pub struct LatestSolidMilestoneChanged(pub Milestone);

pub struct MessageSolidified(pub MessageId);

pub struct TpsMetricsUpdated {
    pub incoming: u64,
    pub new: u64,
    pub known: u64,
    pub invalid: u64,
    pub outgoing: u64,
}

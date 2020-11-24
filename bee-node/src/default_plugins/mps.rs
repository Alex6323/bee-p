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

use crate::plugin::Plugin;

use bee_common_ext::event::Bus;
use bee_protocol::event::TpsMetricsUpdated;

use async_trait::async_trait;
use log::info;

use std::convert::Infallible;

pub struct Mps;

#[async_trait]
impl Plugin for Mps {
    type Config = ();
    type Error = Infallible;

    async fn start(_: Self::Config, bus: &Bus<'_>) -> Result<Self, Self::Error> {
        bus.add_listener::<(), TpsMetricsUpdated, _>(|metrics| {
            info!(
                "incoming {} new {} known {} invalid {} outgoing {}",
                metrics.incoming, metrics.new, metrics.known, metrics.invalid, metrics.outgoing
            );
        });
        Ok(Self)
    }
}

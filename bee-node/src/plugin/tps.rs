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

use bee_common::shutdown::Shutdown;
use bee_common_ext::event::Bus;
use bee_protocol::event::TpsMetricsUpdated;

use log::info;

use std::{convert::Infallible, sync::Arc};

fn tps(metrics: &TpsMetricsUpdated) {
    info!(
        "incoming {} new {} known {} stale {} invalid {} outgoing {}",
        metrics.incoming, metrics.new, metrics.known, metrics.stale, metrics.invalid, metrics.outgoing
    );
}

pub(crate) struct TpsPlugin {}

impl TpsPlugin {
    pub(crate) fn new() -> Self {
        Self {}
    }
}

impl Plugin for TpsPlugin {
    type Error = Infallible;

    fn name(&self) -> &str {
        "tps"
    }

    fn init(&mut self, bus: Arc<Bus>, _: &mut Shutdown) -> Result<(), Self::Error> {
        bus.add_listener(tps);
        Ok(())
    }

    fn start(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

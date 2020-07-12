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

use bee_event::Bus;
use bee_protocol::events::{LatestMilestone, LatestSolidMilestone};

use std::sync::Arc;

fn handle_latest_milestone(latest_milestone: &LatestMilestone) {
    println!("latest milestone");
}

fn handle_latest_solid_milestone(latest_solid_milestone: &LatestSolidMilestone) {
    println!("latest solid milestone");
}

pub fn init(bus: Arc<Bus>) {
    bus.add_listener(handle_latest_milestone);
    bus.add_listener(handle_latest_solid_milestone);
}

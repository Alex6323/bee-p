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

use bee_event::Event;

pub struct LatestMilestone {
    previous: Milestone,
    current: Milestone,
}
impl Event for LatestMilestone {
    fn name() -> &'static str
    where
        Self: Sized,
    {
        "latest_milestone"
    }
}

pub struct LatestSolidMilestone {
    previous: Milestone,
    current: Milestone,
}
impl Event for LatestSolidMilestone {
    fn name() -> &'static str
    where
        Self: Sized,
    {
        "latest_solid_milestone"
    }
}

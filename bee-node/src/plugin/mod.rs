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

use bee_common::shutdown::Shutdown;
use bee_common_ext::event::Bus;

use std::sync::Arc;

mod tps;

pub trait Plugin {
    type Error;

    fn name(&self) -> &str;
    fn init(&mut self, bus: Arc<Bus>, shutdown: &mut Shutdown) -> Result<(), Self::Error>;
    fn start(&mut self) -> Result<(), Self::Error>;
}

pub(crate) fn init(bus: Arc<Bus>, shutdown: &mut Shutdown) {
    tps::TpsPlugin::new().init(bus, shutdown);
}

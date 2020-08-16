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

use bitflags::bitflags;

bitflags! {
    #[derive(Default)]
    pub struct Flags: u8 {
        const SOLID = 0b0000_0001;
        const TAIL = 0b0000_0010;
        const REQUESTED = 0b0000_0100;
        const MILESTONE = 0b0000_1000;
        const CONFIRMED = 0b0001_0000;
    }
}

impl Flags {
    pub fn is_solid(&self) -> bool {
        self.contains(Flags::SOLID)
    }

    pub fn set_solid(&mut self) {
        self.insert(Flags::SOLID);
    }

    pub fn is_tail(&self) -> bool {
        self.contains(Flags::TAIL)
    }

    pub fn set_tail(&mut self) {
        self.insert(Flags::TAIL);
    }

    pub fn is_requested(&self) -> bool {
        self.contains(Flags::REQUESTED)
    }

    pub fn set_requested(&mut self) {
        self.insert(Flags::REQUESTED);
    }

    pub fn is_milestone(&self) -> bool {
        self.contains(Flags::MILESTONE)
    }

    pub fn set_milestone(&mut self) {
        self.insert(Flags::MILESTONE);
    }

    pub fn is_confirmed(&self) -> bool {
        self.contains(Flags::CONFIRMED)
    }

    pub fn set_confirmed(&mut self) {
        self.insert(Flags::CONFIRMED);
    }
}

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

pub(crate) trait OverflowingAdd<Rhs = Self> {
    type Output;

    fn overflowing_add_with_carry(self, other: Rhs, carry: Rhs) -> (Self::Output, bool);
}

impl OverflowingAdd for u32 {
    type Output = Self;

    fn overflowing_add_with_carry(self, other: u32, carry: u32) -> (Self::Output, bool) {
        let (sum, first_overflow) = self.overflowing_add(other);
        let (sum, second_overflow) = sum.overflowing_add(carry);

        (sum, first_overflow | second_overflow)
    }
}

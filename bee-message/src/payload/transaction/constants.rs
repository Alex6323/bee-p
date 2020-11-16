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

use core::ops::Range;

pub const INPUT_OUTPUT_COUNT_MAX: usize = 127;
pub const INPUT_OUTPUT_COUNT_RANGE: Range<usize> = 1..INPUT_OUTPUT_COUNT_MAX + 1;
pub const INPUT_OUTPUT_INDEX_RANGE: Range<u16> = 0..INPUT_OUTPUT_COUNT_MAX as u16;

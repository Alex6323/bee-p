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

use rand::{
    distributions::{uniform::SampleUniform, Distribution, Standard},
    Rng,
};

use std::ops::Range;

pub fn random_integer<T>() -> T
where
    Standard: Distribution<T>,
{
    rand::thread_rng().gen()
}

pub fn random_integer_range<T>(range: Range<T>) -> T
where
    T: SampleUniform,
{
    rand::thread_rng().gen_range(range.start, range.end)
}

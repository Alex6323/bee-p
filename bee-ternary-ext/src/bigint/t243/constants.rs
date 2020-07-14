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

use crate::bigint::{binary_representation::U32Repr, endianness::LittleEndian, T243, U384};

use bee_ternary::{Btrit, Utrit};

use lazy_static::lazy_static;

lazy_static! {
    pub static ref BTRIT_0: T243<Btrit> = T243::<Btrit>::zero();
    pub static ref BTRIT_1: T243<Btrit> = T243::<Btrit>::one();
    pub static ref BTRIT_NEG_1: T243<Btrit> = T243::<Btrit>::neg_one();
    pub static ref UTRIT_0: T243<Utrit> = T243::<Utrit>::zero();
    pub static ref UTRIT_1: T243<Utrit> = T243::<Utrit>::one();
    pub static ref UTRIT_2: T243<Utrit> = T243::<Utrit>::two();
    pub static ref UTRIT_U384_MAX: T243<Utrit> = From::from(U384::<LittleEndian, U32Repr>::max());
    pub static ref UTRIT_U384_MAX_HALF: T243<Utrit> = {
        let mut u384_max = U384::<LittleEndian, U32Repr>::max();
        u384_max.divide_by_two();
        From::from(u384_max)
    };
}

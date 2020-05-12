// Copyright 2020 IOTA Stiftung
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use bee_common::{
    constants::{TRANSACTION_TRIT_LEN as INPUT_LEN, TRANSACTION_TRIT_LEN as TRANS_LEN},
    Trit,
};

pub struct InputTrits(pub(crate) [Trit; INPUT_LEN]);

impl std::ops::Deref for InputTrits {
    type Target = [Trit; TRANS_LEN];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

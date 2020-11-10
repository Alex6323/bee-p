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

use crate::bytes::random_bytes;

use bee_message::{MessageId, MESSAGE_ID_LENGTH};

use std::convert::TryInto;

pub fn random_message_id() -> MessageId {
    MessageId::new(random_bytes(MESSAGE_ID_LENGTH)[..].try_into().unwrap())
}

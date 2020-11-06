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

// #![no_std]

#[macro_use]
extern crate alloc;

mod error;
mod message;
mod message_id;
mod vertex;

pub mod payload;
pub mod prelude;

pub use error::Error;
pub use message::{Message, MessageBuilder};
pub use message_id::{MessageId, MESSAGE_ID_LENGTH};
pub use vertex::Vertex;

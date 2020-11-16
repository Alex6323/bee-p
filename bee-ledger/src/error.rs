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

use bee_message::{payload::transaction::OutputId, Error as MessageError, MessageId};

#[derive(Debug)]
pub enum Error {
    Io(std::io::Error),
    Message(MessageError),
    MissingMessage(MessageId),
    UnsupportedInputType,
    MilestoneMessageNotFound,
    NoMilestonePayload,
    NonContiguousMilestone,
    MerkleProofMismatch,
    InvalidMessagesCount,
    OutputNotFound(OutputId),
    Storage(Box<dyn std::error::Error + Send>),
}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        Error::Io(error)
    }
}

impl From<MessageError> for Error {
    fn from(error: MessageError) -> Self {
        Error::Message(error)
    }
}

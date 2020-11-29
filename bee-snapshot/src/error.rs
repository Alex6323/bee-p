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

use bee_message::Error as MessageError;

#[derive(Debug)]
pub enum Error {
    Io(std::io::Error),
    InvalidVariant,
    InvalidVersion(u8, u8),
    NoDownloadSourceAvailable,
    Message(MessageError),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Io(e) => write!(f, "I/O error happened: {}.", e),
            Error::InvalidVariant => write!(f, "Invalid variant read."),
            Error::InvalidVersion(expected, actual) => write!(f, "Invalid version read: {}, {}.", expected, actual),
            Error::NoDownloadSourceAvailable => write!(f, ""),
            Error::Message(_) => write!(f, ""),
        }
    }
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

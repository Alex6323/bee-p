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

mod message;
mod message_id;

pub mod payload;

pub use message::{Message, MessageBuilder};
pub use message_id::{MessageId, MESSAGE_ID_LENGTH};

use core::fmt;

#[derive(Debug)]
pub enum Error {
    AmountError,
    CountError,
    NoInput,
    NoOutput,
    DuplicateError,
    // TODO add index
    InvalidIndex,
    InvalidAddress,
    InvalidSignature,
    OrderError,
    HashError,
    PathError,
    MissingField(&'static str),
    SigningError(bee_signing_ext::binary::Error),
    SignatureError(bee_signing_ext::SignatureError),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::AmountError => write!(f, "Invalid amount provided."),
            Error::CountError => write!(f, "Invalid count number provided."),
            Error::NoInput => write!(f, "No input provided."),
            Error::NoOutput => write!(f, "No output provided."),
            Error::DuplicateError => write!(f, "The object in the set must be unique."),
            Error::InvalidIndex => write!(f, "Invalid index provided."),
            Error::InvalidAddress => write!(f, "Invalid address provided."),
            Error::InvalidSignature => write!(f, "Invalid signature provided."),
            Error::OrderError => write!(f, "The vector is not sorted by lexicographical order."),
            Error::HashError => write!(f, "The format of provided hash is not correct."),
            Error::PathError => write!(f, "The format of provided BIP32 path is not correct."),
            Error::MissingField(s) => write!(f, "Missing required field: {}.", s),
            Error::SigningError(e) => write!(f, "{}", e),
            Error::SignatureError(e) => write!(f, "{}", e),
        }
    }
}

impl From<bee_signing_ext::binary::Error> for Error {
    fn from(error: bee_signing_ext::binary::Error) -> Self {
        Error::SigningError(error)
    }
}

impl From<bee_signing_ext::SignatureError> for Error {
    fn from(error: bee_signing_ext::SignatureError) -> Self {
        Error::SignatureError(error)
    }
}

use std::io::prelude::*;

trait WriteBytes {
    fn len_bytes(&self) -> usize;

    fn write_bytes(&self, buffer: &mut Vec<u8>);
}

impl WriteBytes for u8 {
    fn len_bytes(&self) -> usize {
        1
    }

    fn write_bytes(&self, buffer: &mut Vec<u8>) {
        self.to_be_bytes().as_ref().write_bytes(buffer);
    }
}

impl WriteBytes for u16 {
    fn len_bytes(&self) -> usize {
        2
    }

    fn write_bytes(&self, buffer: &mut Vec<u8>) {
        self.to_be_bytes().as_ref().write_bytes(buffer);
    }
}

impl WriteBytes for u32 {
    fn len_bytes(&self) -> usize {
        4
    }

    fn write_bytes(&self, buffer: &mut Vec<u8>) {
        self.to_be_bytes().as_ref().write_bytes(buffer);
    }
}

impl WriteBytes for u64 {
    fn len_bytes(&self) -> usize {
        8
    }

    fn write_bytes(&self, buffer: &mut Vec<u8>) {
        self.to_be_bytes().as_ref().write_bytes(buffer);
    }
}

impl WriteBytes for &[u8] {
    fn len_bytes(&self) -> usize {
        self.len()
    }

    fn write_bytes(&self, buffer: &mut Vec<u8>) {
        buffer.write_all(self).unwrap();
    }
}

impl<T: WriteBytes + ?Sized> WriteBytes for Box<T> {
    fn len_bytes(&self) -> usize {
        self.as_ref().len_bytes()
    }

    fn write_bytes(&self, buffer: &mut Vec<u8>) {
        self.as_ref().write_bytes(buffer)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let msg = Message::builder()
            .parent1(MessageId::new([
                0xF5, 0x32, 0xA5, 0x35, 0x45, 0x10, 0x32, 0x76, 0xB4, 0x68, 0x76, 0xC4, 0x73, 0x84, 0x6D, 0x98, 0x64,
                0x8E, 0xE4, 0x18, 0x46, 0x8B, 0xCE, 0x76, 0xDF, 0x48, 0x68, 0x64, 0x8D, 0xD7, 0x3E, 0x5D,
            ]))
            .parent2(MessageId::new([
                0x78, 0xD5, 0x46, 0xB4, 0x6A, 0xEC, 0x45, 0x57, 0x87, 0x21, 0x39, 0xA4, 0x8F, 0x66, 0xBC, 0x56, 0x76,
                0x87, 0xE8, 0x41, 0x35, 0x78, 0xA1, 0x43, 0x23, 0x54, 0x87, 0x32, 0x35, 0x89, 0x14, 0xA2,
            ]))
            .payload(payload::Payload::Indexation(Box::new(payload::Indexation::new(
                "0000".to_owned(),
                Box::new([0x48, 0x65, 0x6c, 0x6c, 0x6f, 0x20, 0x49, 0x6f, 0x74, 0x61]),
            ))))
            .build()
            .unwrap();

        let mut buffer = Vec::with_capacity(1000);

        msg.write_bytes(&mut buffer);

        println!("{:x?}", buffer);
    }
}

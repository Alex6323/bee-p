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

use crate::{header::SnapshotHeader, local::LocalSnapshot, metadata::SnapshotMetadata};

use bee_message::prelude::MessageId;

use std::collections::HashSet;

use std::{
    fs::OpenOptions,
    io::{BufReader, BufWriter},
};

// TODO detail errors
#[derive(Debug)]
pub enum Error {
    IOError(std::io::Error),
    InvalidVersion(u8, u8),
    InvalidMilestoneHash,
    InvalidSolidEntryPointHash,
    InvalidAddress,
    InvalidSupply(u64, u64),
}
impl LocalSnapshot {
    pub fn from_file(path: &str) -> Result<LocalSnapshot, Error> {
        let mut _reader = BufReader::new(OpenOptions::new().read(true).open(path).map_err(Error::IOError)?);

        Ok(LocalSnapshot {
            metadata: SnapshotMetadata {
                header: SnapshotHeader {
                    timestamp: 0,
                    coordinator: [0; 32],
                    sep_index: 0,
                    sep_id: MessageId::null(),
                    ledger_index: 0,
                    ledger_id: MessageId::null(),
                },
                solid_entry_points: HashSet::new(),
            },
        })
    }

    pub fn to_file(&self, path: &str) -> Result<(), Error> {
        let mut _writer = BufWriter::new(
            OpenOptions::new()
                .write(true)
                .truncate(true)
                .create(true)
                .open(path)
                .map_err(Error::IOError)?,
        );

        Ok(())
    }
}

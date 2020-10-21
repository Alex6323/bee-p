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

use crate::{header::SnapshotHeader, kind::Kind, metadata::SnapshotMetadata};

use bee_common_ext::packable::{Error as PackableError, Packable, Read, Write};
use bee_message::prelude::MessageId;

use log::{error, info};

use std::{
    collections::HashSet,
    fs::OpenOptions,
    io::{BufReader, BufWriter},
};

// TODO remove ?
#[derive(Debug)]
pub enum Error {
    IOError(std::io::Error),
}

pub struct LocalSnapshot {
    pub(crate) metadata: SnapshotMetadata,
}

impl LocalSnapshot {
    pub fn metadata(&self) -> &SnapshotMetadata {
        &self.metadata
    }

    pub fn from_file(path: &str) -> Result<LocalSnapshot, Error> {
        let mut reader = BufReader::new(OpenOptions::new().read(true).open(path).map_err(Error::IOError)?);

        // TODO unwrap
        Ok(LocalSnapshot::unpack(&mut reader).unwrap())
    }

    pub fn to_file(&self, path: &str) -> Result<(), Error> {
        let mut writer = BufWriter::new(
            OpenOptions::new()
                .write(true)
                .truncate(true)
                .create(true)
                .open(path)
                .map_err(Error::IOError)?,
        );

        // TODO unwrap
        self.pack(&mut writer).unwrap();

        Ok(())
    }
}

impl Packable for LocalSnapshot {
    fn packed_len(&self) -> usize {
        self.metadata.packed_len()
    }

    fn pack<W: Write>(&self, buf: &mut W) -> Result<(), PackableError> {
        self.metadata.pack(buf)?;

        Ok(())
    }

    fn unpack<R: Read + ?Sized>(buf: &mut R) -> Result<Self, PackableError>
    where
        Self: Sized,
    {
        Ok(Self {
            metadata: SnapshotMetadata::unpack(buf)?,
        })
    }
}

#[allow(dead_code)] // TODO: When pruning is enabled
pub(crate) fn snapshot(path: &str, index: u32) -> Result<(), Error> {
    info!("Creating local snapshot at index {}...", index);

    let ls = LocalSnapshot {
        metadata: SnapshotMetadata {
            header: SnapshotHeader {
                kind: Kind::Full,
                timestamp: 0,
                coordinator: [0; 32],
                sep_index: 0,
                sep_id: MessageId::null(),
                ledger_index: 0,
                ledger_id: MessageId::null(),
            },
            solid_entry_points: HashSet::new(),
        },
    };

    let file = path.to_string() + "_tmp";

    if let Err(e) = ls.to_file(&file) {
        error!("Failed to write local snapshot to file {}: {:?}.", file, e);
    }

    info!("Created local snapshot at index {}.", index);

    Ok(())
}

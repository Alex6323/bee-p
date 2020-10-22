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

use crate::{header::SnapshotHeader, kind::Kind, output::Output};

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
    pub(crate) header: SnapshotHeader,
    pub(crate) solid_entry_points: HashSet<MessageId>,
    pub(crate) outputs: Vec<Output>,
}

impl LocalSnapshot {
    pub fn header(&self) -> &SnapshotHeader {
        &self.header
    }

    pub fn solid_entry_points(&self) -> &HashSet<MessageId> {
        &self.solid_entry_points
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
        self.header.packed_len()
        // + TODO SEP
    }

    fn pack<W: Write>(&self, buf: &mut W) -> Result<(), PackableError> {
        self.header.pack(buf)?;
        // + TODO SEP

        Ok(())
    }

    // TODO stream ?
    fn unpack<R: Read + ?Sized>(buf: &mut R) -> Result<Self, PackableError>
    where
        Self: Sized,
    {
        let header = SnapshotHeader::unpack(buf)?;

        let sep_count = u64::unpack(buf)? as usize;
        let output_count = if header.kind() == Kind::Full {
            u64::unpack(buf)? as usize
        } else {
            0
        };
        let milestone_diff_count = u64::unpack(buf)? as usize;

        let mut solid_entry_points = HashSet::with_capacity(sep_count);
        for _ in 0..sep_count {
            solid_entry_points.insert(MessageId::unpack(buf)?);
        }

        let mut outputs = Vec::with_capacity(output_count);
        if header.kind() == Kind::Full {
            for _ in 0..output_count {
                outputs.push(Output::unpack(buf)?);
            }
        }

        for _ in 0..milestone_diff_count {}

        // TODO SEP
        Ok(Self {
            header,
            solid_entry_points,
            outputs,
        })
    }
}

#[allow(dead_code)] // TODO: When pruning is enabled
pub(crate) fn snapshot(path: &str, index: u32) -> Result<(), Error> {
    info!("Creating local snapshot at index {}...", index);

    let ls = LocalSnapshot {
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
        outputs: Vec::new(),
    };

    let file = path.to_string() + "_tmp";

    if let Err(e) = ls.to_file(&file) {
        error!("Failed to write local snapshot to file {}: {:?}.", file, e);
    }

    info!("Created local snapshot at index {}.", index);

    Ok(())
}

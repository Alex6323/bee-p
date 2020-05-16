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

use bee_bundle::{Hash, TransactionField};
use bee_ternary::{T1B1Buf, TryteBuf};

use std::{
    fs::File,
    io::{BufRead, BufReader},
};

#[derive(Debug)]
pub enum SnapshotMetadataError {
    IOError(std::io::Error),
    InvalidHash,
    InvalidIndex(std::num::ParseIntError),
    InvalidTimestamp(std::num::ParseIntError),
    InvalidSolidEntryPointsNumber(std::num::ParseIntError),
    InvalidSeenMilestonesNumber(std::num::ParseIntError),
    InvalidSolidEntryPointHash,
    InvalidSeenMilestoneHash,
}

// TODO use a Hash type instead of TritBuf<T1B1Buf>
pub struct SnapshotMetadata {
    hash: Hash,
    index: u32,
    timestamp: u64,
    solid_entry_points: Vec<Hash>,
    seen_milestones: Vec<Hash>,
}

impl SnapshotMetadata {
    pub fn new(path: &str) -> Result<Self, SnapshotMetadataError> {
        match File::open(path) {
            Ok(file) => {
                let mut reader = BufReader::new(file);
                let mut line = String::new();

                // Parsing snapshot metadata hash
                let hash = match reader.read_line(&mut line) {
                    Ok(_) => match TryteBuf::try_from_str(&line[..line.len() - 1]) {
                        Ok(buf) => Hash::try_from_inner(buf.as_trits().encode::<T1B1Buf>())
                            .map_err(|_| SnapshotMetadataError::InvalidHash),
                        Err(_) => Err(SnapshotMetadataError::InvalidHash),
                    },
                    Err(e) => Err(SnapshotMetadataError::IOError(e)),
                }?;

                // Parsing snapshot metadata index
                line.clear();
                let index = match reader.read_line(&mut line) {
                    Ok(_) => line[..line.len() - 1]
                        .parse::<u32>()
                        .map_err(SnapshotMetadataError::InvalidIndex),
                    Err(e) => Err(SnapshotMetadataError::IOError(e)),
                }?;

                // Parsing snapshot metadata timestamp
                line.clear();
                let timestamp = match reader.read_line(&mut line) {
                    Ok(_) => line[..line.len() - 1]
                        .parse::<u64>()
                        .map_err(SnapshotMetadataError::InvalidTimestamp),
                    Err(e) => Err(SnapshotMetadataError::IOError(e)),
                }?;

                // Parsing solid entry points number
                line.clear();
                let solid_entry_points_num = match reader.read_line(&mut line) {
                    Ok(_) => line[..line.len() - 1]
                        .parse::<usize>()
                        .map_err(SnapshotMetadataError::InvalidSolidEntryPointsNumber),
                    Err(e) => Err(SnapshotMetadataError::IOError(e)),
                }?;

                // Parsing seen milestones number
                line.clear();
                let seen_milestones_num = match reader.read_line(&mut line) {
                    Ok(_) => line[..line.len() - 1]
                        .parse::<usize>()
                        .map_err(SnapshotMetadataError::InvalidSeenMilestonesNumber),
                    Err(e) => Err(SnapshotMetadataError::IOError(e)),
                }?;

                // Parsing solid entry points
                let mut solid_entry_points = Vec::with_capacity(solid_entry_points_num);
                for _ in 0..solid_entry_points_num {
                    line.clear();
                    let hash = match reader.read_line(&mut line) {
                        Ok(_) => {
                            let tokens: Vec<&str> = line.split(';').collect();
                            // TODO check size of tokens
                            // TODO what to do with index ?
                            match TryteBuf::try_from_str(&tokens[0][..tokens[0].len()]) {
                                Ok(buf) => Hash::try_from_inner(buf.as_trits().encode::<T1B1Buf>())
                                    .map_err(|_| SnapshotMetadataError::InvalidSolidEntryPointHash),
                                Err(_) => Err(SnapshotMetadataError::InvalidSolidEntryPointHash),
                            }
                        }
                        Err(e) => Err(SnapshotMetadataError::IOError(e)),
                    }?;
                    solid_entry_points.push(hash);
                }

                // Parsing seen milestones
                let mut seen_milestones = Vec::with_capacity(seen_milestones_num);
                for _ in 0..seen_milestones_num {
                    line.clear();
                    let hash = match reader.read_line(&mut line) {
                        Ok(_) => {
                            let tokens: Vec<&str> = line.split(';').collect();
                            // TODO check size of tokens
                            // TODO what to do with index ?
                            match TryteBuf::try_from_str(&tokens[0][..tokens[0].len()]) {
                                Ok(buf) => Hash::try_from_inner(buf.as_trits().encode::<T1B1Buf>())
                                    .map_err(|_| SnapshotMetadataError::InvalidSeenMilestoneHash),
                                Err(_) => Err(SnapshotMetadataError::InvalidSeenMilestoneHash),
                            }
                        }
                        Err(e) => Err(SnapshotMetadataError::IOError(e)),
                    }?;
                    seen_milestones.push(hash);
                }

                Ok(Self {
                    hash,
                    index,
                    timestamp,
                    solid_entry_points,
                    seen_milestones,
                })
            }
            Err(e) => Err(SnapshotMetadataError::IOError(e)),
        }
    }

    pub fn hash(&self) -> &Hash {
        &self.hash
    }

    pub fn index(&self) -> u32 {
        self.index
    }

    pub fn timestamp(&self) -> u64 {
        self.timestamp
    }

    pub fn solid_entry_points(&self) -> &Vec<Hash> {
        &self.solid_entry_points
    }

    pub fn seen_milestones(&self) -> &Vec<Hash> {
        &self.seen_milestones
    }
}

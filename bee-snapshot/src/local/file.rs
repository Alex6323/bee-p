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

use crate::{constants::IOTA_SUPPLY, header::SnapshotHeader, local::LocalSnapshot, metadata::SnapshotMetadata};

use bytemuck::cast_slice;
use log::debug;

use std::{
    collections::HashMap,
    fs::OpenOptions,
    io::{BufReader, BufWriter, Read, Write},
};

const VERSION: u8 = 4;

// TODO detail errors
#[derive(Debug)]
pub enum Error {
    IOError(std::io::Error),
    InvalidVersion(u8, u8),
    InvalidMilestoneHash,
    InvalidSolidEntryPointHash,
    InvalidSeenMilestoneHash,
    InvalidAddress,
    InvalidSupply(u64, u64),
}
impl LocalSnapshot {
    pub fn from_file(path: &str) -> Result<LocalSnapshot, Error> {
        let mut reader = BufReader::new(OpenOptions::new().read(true).open(path).map_err(Error::IOError)?);

        // Version byte

        let mut buf = [0u8];
        let version = match reader.read_exact(&mut buf) {
            Ok(_) => buf[0],
            Err(e) => return Err(Error::IOError(e)),
        };

        if version != VERSION {
            return Err(Error::InvalidVersion(version, VERSION));
        }

        debug!("Version: {}.", version);

        // Milestone hash

        let mut buf = [0u8; 49];
        let hash = match reader.read_exact(&mut buf) {
            Ok(_) => match Trits::<T5B1>::try_from_raw(cast_slice(&buf), HASH_LENGTH) {
                Ok(trits) => Hash::try_from_inner(trits.encode::<T1B1Buf>()).map_err(|_| Error::InvalidMilestoneHash),
                Err(_) => Err(Error::InvalidMilestoneHash),
            },
            Err(e) => Err(Error::IOError(e)),
        }?;

        debug!("Hash: {}.", hash.iter_trytes().map(char::from).collect::<String>());

        // Milestone index

        let mut buf = [0u8; std::mem::size_of::<u32>()];
        let index = match reader.read_exact(&mut buf) {
            Ok(_) => u32::from_le_bytes(buf),
            Err(e) => return Err(Error::IOError(e)),
        };

        debug!("Index: {}.", index);

        // Timestamp

        let mut buf = [0u8; std::mem::size_of::<u64>()];
        let timestamp = match reader.read_exact(&mut buf) {
            Ok(_) => u64::from_le_bytes(buf),
            Err(e) => return Err(Error::IOError(e)),
        };

        debug!("Timestamp: {}.", timestamp);

        // Number of solid entry points

        let mut buf = [0u8; std::mem::size_of::<u32>()];
        let solid_entry_points_num = match reader.read_exact(&mut buf) {
            Ok(_) => u32::from_le_bytes(buf),
            Err(e) => return Err(Error::IOError(e)),
        };

        debug!("Solid entry points: {}.", solid_entry_points_num);

        // Number of seen milestones

        let mut buf = [0u8; std::mem::size_of::<u32>()];
        let seen_milestones_num = match reader.read_exact(&mut buf) {
            Ok(_) => u32::from_le_bytes(buf),
            Err(e) => return Err(Error::IOError(e)),
        };

        debug!("Seen milestones: {}.", seen_milestones_num);

        // Number of balances

        let mut buf = [0u8; std::mem::size_of::<u32>()];
        let balances_num = match reader.read_exact(&mut buf) {
            Ok(_) => u32::from_le_bytes(buf),
            Err(e) => return Err(Error::IOError(e)),
        };

        debug!("Balances: {}.", balances_num);

        // Number of spent addresses

        let mut buf = [0u8; std::mem::size_of::<u32>()];
        let spent_addresses_num = match reader.read_exact(&mut buf) {
            Ok(_) => u32::from_le_bytes(buf),
            Err(e) => return Err(Error::IOError(e)),
        };

        debug!("Spent addresses: {}.", spent_addresses_num);

        // Solid entry points

        let mut buf_hash = [0u8; 49];
        let mut buf_index = [0u8; std::mem::size_of::<u32>()];
        let mut solid_entry_points = HashMap::with_capacity(solid_entry_points_num as usize);
        for _ in 0..solid_entry_points_num {
            let hash = match reader.read_exact(&mut buf_hash) {
                Ok(_) => match Trits::<T5B1>::try_from_raw(cast_slice(&buf_hash), HASH_LENGTH) {
                    Ok(trits) => {
                        Hash::try_from_inner(trits.encode::<T1B1Buf>()).map_err(|_| Error::InvalidSolidEntryPointHash)
                    }
                    Err(_) => Err(Error::InvalidSolidEntryPointHash),
                },
                Err(e) => Err(Error::IOError(e)),
            }?;
            let index = match reader.read_exact(&mut buf_index) {
                Ok(_) => u32::from_le_bytes(buf_index),
                Err(e) => return Err(Error::IOError(e)),
            };
            solid_entry_points.insert(hash, index);
        }

        // Seen milestones

        let mut buf_hash = [0u8; 49];
        let mut buf_index = [0u8; std::mem::size_of::<u32>()];
        let mut seen_milestones = HashMap::with_capacity(seen_milestones_num as usize);
        for _ in 0..seen_milestones_num {
            let seen_milestone = match reader.read_exact(&mut buf_hash) {
                Ok(_) => match Trits::<T5B1>::try_from_raw(cast_slice(&buf_hash), HASH_LENGTH) {
                    Ok(trits) => {
                        Hash::try_from_inner(trits.encode::<T1B1Buf>()).map_err(|_| Error::InvalidSeenMilestoneHash)
                    }
                    Err(_) => Err(Error::InvalidSeenMilestoneHash),
                },
                Err(e) => Err(Error::IOError(e)),
            }?;
            let index = match reader.read_exact(&mut buf_index) {
                Ok(_) => u32::from_le_bytes(buf_index),
                Err(e) => return Err(Error::IOError(e)),
            };
            seen_milestones.insert(seen_milestone, index);
        }

        // Balances

        let mut buf_address = [0u8; 49];
        let mut buf_value = [0u8; std::mem::size_of::<u64>()];
        let mut state = HashMap::with_capacity(balances_num as usize);
        let mut supply: u64 = 0;
        for i in 0..balances_num {
            let address = match reader.read_exact(&mut buf_address) {
                Ok(_) => match Trits::<T5B1>::try_from_raw(cast_slice(&buf_address), HASH_LENGTH) {
                    Ok(trits) => Address::try_from_inner(trits.encode::<T1B1Buf>()).map_err(|_| Error::InvalidAddress),
                    Err(_) => Err(Error::InvalidAddress),
                },
                Err(e) => Err(Error::IOError(e)),
            }?;
            let value = match reader.read_exact(&mut buf_value) {
                Ok(_) => u64::from_le_bytes(buf_value),
                Err(e) => return Err(Error::IOError(e)),
            };

            if i % 50_000 == 0 && i != 0 {
                debug!(
                    "Read {}/{} ({:.0}%) balances.",
                    i,
                    balances_num,
                    ((i * 100) as f64) / (balances_num as f64)
                );
            }

            state.insert(address, value);
            supply += value;
        }

        if supply != IOTA_SUPPLY {
            return Err(Error::InvalidSupply(supply, IOTA_SUPPLY));
        }

        // TODO hash ?

        Ok(LocalSnapshot {
            metadata: SnapshotMetadata {
                header: SnapshotHeader {
                    coordinator: Hash::zeros(),
                    hash,
                    snapshot_index: index,
                    entry_point_index: index,
                    pruning_index: index,
                    timestamp,
                },
                solid_entry_points,
                seen_milestones,
            },
            state,
        })
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

        // Version byte

        if let Err(e) = writer.write_all(&[VERSION]) {
            return Err(Error::IOError(e));
        };

        // Milestone hash

        if let Err(e) = writer.write_all(&cast_slice(
            self.metadata.header.hash.to_inner().encode::<T5B1Buf>().as_i8_slice(),
        )) {
            return Err(Error::IOError(e));
        }

        // Milestone index

        if let Err(e) = writer.write_all(&self.metadata.header.snapshot_index.to_le_bytes()) {
            return Err(Error::IOError(e));
        }

        // Timestamp

        if let Err(e) = writer.write_all(&self.metadata.header.timestamp.to_le_bytes()) {
            return Err(Error::IOError(e));
        }

        // Number of solid entry points

        if let Err(e) = writer.write_all(&(self.metadata.solid_entry_points.len() as u32).to_le_bytes()) {
            return Err(Error::IOError(e));
        }

        // Number of seen milestones

        if let Err(e) = writer.write_all(&(self.metadata.seen_milestones.len() as u32).to_le_bytes()) {
            return Err(Error::IOError(e));
        }

        // Number of balances

        if let Err(e) = writer.write_all(&(self.state.len() as u32).to_le_bytes()) {
            return Err(Error::IOError(e));
        }

        // Number of spent addresses

        if let Err(e) = writer.write_all(&0u32.to_le_bytes()) {
            return Err(Error::IOError(e));
        }

        // Solid entry points

        for (hash, index) in self.metadata.solid_entry_points.iter() {
            if let Err(e) = writer.write_all(&cast_slice(hash.as_trits().encode::<T5B1Buf>().as_i8_slice())) {
                return Err(Error::IOError(e));
            }
            if let Err(e) = writer.write_all(&index.to_le_bytes()) {
                return Err(Error::IOError(e));
            }
        }

        // Seen milestones

        for (hash, index) in self.metadata.seen_milestones.iter() {
            if let Err(e) = writer.write_all(&cast_slice(hash.as_trits().encode::<T5B1Buf>().as_i8_slice())) {
                return Err(Error::IOError(e));
            }
            if let Err(e) = writer.write_all(&index.to_le_bytes()) {
                return Err(Error::IOError(e));
            }
        }

        // Balances

        for (address, balance) in self.state.iter() {
            if let Err(e) = writer.write_all(&cast_slice(address.to_inner().encode::<T5B1Buf>().as_i8_slice())) {
                return Err(Error::IOError(e));
            }
            if let Err(e) = writer.write_all(&balance.to_le_bytes()) {
                return Err(Error::IOError(e));
            }
        }

        // TODO hash ?

        Ok(())
    }
}

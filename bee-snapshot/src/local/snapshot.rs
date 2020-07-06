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

use crate::{constants::IOTA_SUPPLY, local::LocalSnapshotMetadata, state::SnapshotState};

use bee_crypto::ternary::Hash;
use bee_ternary::{T1B1Buf, Trits, T5B1};
use bee_transaction::bundled::{Address, BundledTransactionField};

use std::collections::HashMap;

use async_std::{fs::File, prelude::*};
use bytemuck::cast_slice;
use log::info;

pub struct LocalSnapshot {
    metadata: LocalSnapshotMetadata,
    state: SnapshotState,
}

const VERSION: u8 = 4;

// TODO detail errors
#[derive(Debug)]
pub enum Error {
    IOError(async_std::io::Error),
    InvalidVersion,
    InvalidMilestoneHash,
    InvalidMilestoneIndex,
    NonZeroSpentAddressesNumber,
    InvalidSolidEntryPointHash,
    InvalidSeenMilestoneHash,
    InvalidAddress,
    InvalidSupply,
}
impl LocalSnapshot {
    pub async fn from_file(path: &str) -> Result<LocalSnapshot, Error> {
        // TODO BufReader ?
        let mut file = File::open(path).await.map_err(|e| Error::IOError(e))?;

        // Version byte

        let mut buf = [0u8];
        match file.read_exact(&mut buf).await {
            Ok(_) => {
                if buf[0] != VERSION {
                    return Err(Error::InvalidVersion);
                }
            }
            Err(e) => return Err(Error::IOError(e)),
        };

        // Milestone hash

        let mut buf = [0u8; 49];
        let hash = match file.read_exact(&mut buf).await {
            Ok(_) => match Trits::<T5B1>::try_from_raw(cast_slice(&buf), 243) {
                Ok(trits) => Hash::try_from_inner(trits.encode::<T1B1Buf>()).map_err(|_| Error::InvalidMilestoneHash),
                Err(_) => Err(Error::InvalidMilestoneHash),
            },
            Err(e) => Err(Error::IOError(e)),
        }?;

        // Milestone index

        let mut buf = [0u8; std::mem::size_of::<u32>()];
        let index = match file.read_exact(&mut buf).await {
            Ok(_) => u32::from_le_bytes(buf),
            Err(e) => return Err(Error::IOError(e)),
        };

        // Timestamp

        let mut buf = [0u8; std::mem::size_of::<u64>()];
        let timestamp = match file.read_exact(&mut buf).await {
            Ok(_) => u64::from_le_bytes(buf),
            Err(e) => return Err(Error::IOError(e)),
        };

        // Number of solid entry points

        let mut buf = [0u8; std::mem::size_of::<u32>()];
        let solid_entry_points_num = match file.read_exact(&mut buf).await {
            Ok(_) => u32::from_le_bytes(buf),
            Err(e) => return Err(Error::IOError(e)),
        };

        // Number of seen milestones

        let mut buf = [0u8; std::mem::size_of::<u32>()];
        let seen_milestones_num = match file.read_exact(&mut buf).await {
            Ok(_) => u32::from_le_bytes(buf),
            Err(e) => return Err(Error::IOError(e)),
        };

        // Number of balances

        let mut buf = [0u8; std::mem::size_of::<u32>()];
        let balances_num = match file.read_exact(&mut buf).await {
            Ok(_) => u32::from_le_bytes(buf),
            Err(e) => return Err(Error::IOError(e)),
        };

        // Number of spent addresses

        let mut buf = [0u8; std::mem::size_of::<u32>()];
        match file.read_exact(&mut buf).await {
            Ok(_) => {
                if u32::from_le_bytes(buf) != 0 {
                    return Err(Error::NonZeroSpentAddressesNumber);
                }
            }
            Err(e) => return Err(Error::IOError(e)),
        };

        // Solid entry points

        let mut buf_hash = [0u8; 49];
        let mut buf_index = [0u8; std::mem::size_of::<u32>()];
        let mut solid_entry_points = HashMap::with_capacity(solid_entry_points_num as usize);
        for _ in 0..solid_entry_points_num {
            let hash = match file.read_exact(&mut buf_hash).await {
                Ok(_) => match Trits::<T5B1>::try_from_raw(cast_slice(&buf_hash), 243) {
                    Ok(trits) => {
                        Hash::try_from_inner(trits.encode::<T1B1Buf>()).map_err(|_| Error::InvalidSolidEntryPointHash)
                    }
                    Err(_) => Err(Error::InvalidSolidEntryPointHash),
                },
                Err(e) => Err(Error::IOError(e)),
            }?;
            let index = match file.read_exact(&mut buf_index).await {
                Ok(_) => u32::from_le_bytes(buf_index),
                Err(e) => return Err(Error::IOError(e)),
            };
            solid_entry_points.insert(hash, index);
        }

        // Seen milestones

        let mut buf_hash = [0u8; 49];
        let mut buf_index = [0u8; std::mem::size_of::<u32>()];
        let mut seen_milestones = Vec::with_capacity(seen_milestones_num as usize);
        for _ in 0..seen_milestones_num {
            let seen_milestone = match file.read_exact(&mut buf_hash).await {
                Ok(_) => match Trits::<T5B1>::try_from_raw(cast_slice(&buf_hash), 243) {
                    Ok(trits) => {
                        Hash::try_from_inner(trits.encode::<T1B1Buf>()).map_err(|_| Error::InvalidSeenMilestoneHash)
                    }
                    Err(_) => Err(Error::InvalidSeenMilestoneHash),
                },
                Err(e) => Err(Error::IOError(e)),
            }?;
            seen_milestones.push(seen_milestone);
            // TODO should we use that ?
            match file.read_exact(&mut buf_index).await {
                Ok(_) => u32::from_le_bytes(buf_index),
                Err(e) => return Err(Error::IOError(e)),
            };
        }

        // amountOfBalances * balance:value - 49 bytes + int64

        let mut buf_address = [0u8; 49];
        let mut buf_value = [0u8; std::mem::size_of::<u64>()];
        let mut state = SnapshotState::with_capacity(balances_num as usize);
        let mut supply: u64 = 0;
        for i in 0..balances_num {
            let address = match file.read_exact(&mut buf_address).await {
                Ok(_) => match Trits::<T5B1>::try_from_raw(cast_slice(&buf_address), 243) {
                    Ok(trits) => Address::try_from_inner(trits.encode::<T1B1Buf>()).map_err(|_| Error::InvalidAddress),
                    Err(_) => Err(Error::InvalidAddress),
                },
                Err(e) => Err(Error::IOError(e)),
            }?;
            let value = match file.read_exact(&mut buf_value).await {
                Ok(_) => u64::from_le_bytes(buf_value),
                Err(e) => return Err(Error::IOError(e)),
            };

            if i % 10_000 == 0 && i != 0 {
                info!(
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
            return Err(Error::InvalidSupply);
        }

        // TODO spend addresses ?
        // TODO hash

        Ok(LocalSnapshot {
            metadata: LocalSnapshotMetadata {
                hash,
                index,
                timestamp,
                solid_entry_points,
                seen_milestones,
            },
            state,
        })
    }

    pub fn metadata(&self) -> &LocalSnapshotMetadata {
        &self.metadata
    }

    pub fn state(&self) -> &SnapshotState {
        &self.state
    }

    pub fn into_state(self) -> SnapshotState {
        self.state
    }
}

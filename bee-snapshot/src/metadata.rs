use bee_bundle::{
    Hash,
    TransactionField,
};
use bee_ternary::{
    T1B1Buf,
    TritBuf,
    TryteBuf,
};

use std::{
    fs::File,
    io::{
        BufRead,
        BufReader,
    },
};

#[derive(Debug)]
pub enum SnapshotMetadataError {
    IOError(std::io::Error),
    InvalidSnapshotMetadataHash,
    InvalidSnapshotMetadataIndex(std::num::ParseIntError),
    InvalidSnapshotMetadataTimestamp(std::num::ParseIntError),
    InvalidSolidEntryPointNumber(std::num::ParseIntError),
    InvalidSeenMilestoneNumber(std::num::ParseIntError),
    InvalidSolidEntryPointHash,
    InvalidSeenMilestoneHash,
}

// TODO use a Hash type instead of TritBuf<T1B1Buf>
pub struct SnapshotMetadata {
    hash: TritBuf<T1B1Buf>,
    index: u64,
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
                // TODO check read_line return value

                // Parsing snapshot metadata hash
                // TODO check trytes size
                let hash = match reader.read_line(&mut line) {
                    Ok(_) => match TryteBuf::try_from_str(&line[..line.len() - 1]) {
                        Ok(buf) => buf.as_trits().encode::<T1B1Buf>(),
                        Err(_) => Err(SnapshotMetadataError::InvalidSnapshotMetadataHash)?,
                    },
                    Err(e) => Err(SnapshotMetadataError::IOError(e))?,
                };

                // Parsing snapshot metadata index
                line.clear();
                let index = match reader.read_line(&mut line) {
                    Ok(_) => line[..line.len() - 1]
                        .parse::<u64>()
                        .map_err(|e| SnapshotMetadataError::InvalidSnapshotMetadataIndex(e)),
                    Err(e) => Err(SnapshotMetadataError::IOError(e)),
                }?;

                // Parsing snapshot metadata timestamp
                line.clear();
                let timestamp = match reader.read_line(&mut line) {
                    Ok(_) => line[..line.len() - 1]
                        .parse::<u64>()
                        .map_err(|e| SnapshotMetadataError::InvalidSnapshotMetadataTimestamp(e)),
                    Err(e) => Err(SnapshotMetadataError::IOError(e)),
                }?;

                // Parsing solid entry points number
                line.clear();
                let solid_entry_points_num = match reader.read_line(&mut line) {
                    Ok(_) => line[..line.len() - 1]
                        .parse::<usize>()
                        .map_err(|e| SnapshotMetadataError::InvalidSolidEntryPointNumber(e)),
                    Err(e) => Err(SnapshotMetadataError::IOError(e)),
                }?;

                // Parsing seen milestones number
                line.clear();
                let seen_milestones_num = match reader.read_line(&mut line) {
                    Ok(_) => line[..line.len() - 1]
                        .parse::<usize>()
                        .map_err(|e| SnapshotMetadataError::InvalidSeenMilestoneNumber(e)),
                    Err(e) => Err(SnapshotMetadataError::IOError(e)),
                }?;

                // Parsing solid entry points
                let mut solid_entry_points = Vec::with_capacity(solid_entry_points_num);
                for _ in 0..solid_entry_points_num {
                    line.clear();
                    let hash = match reader.read_line(&mut line) {
                        Ok(_) => {
                            let tokens: Vec<&str> = line.split(";").collect();
                            // TODO check size of tokens
                            // TODO what to do with index ?
                            // TODO check trytes size
                            match TryteBuf::try_from_str(&tokens[0][..tokens[0].len()]) {
                                Ok(buf) => Hash::try_from_inner(buf.as_trits().encode::<T1B1Buf>())
                                    .map_err(|_| SnapshotMetadataError::InvalidSolidEntryPointHash)?,
                                Err(_) => Err(SnapshotMetadataError::InvalidSolidEntryPointHash)?,
                            }
                        }
                        Err(e) => Err(SnapshotMetadataError::IOError(e))?,
                    };
                    solid_entry_points.push(hash);
                }

                // Parsing seen milestones
                let mut seen_milestones = Vec::with_capacity(seen_milestones_num);
                for _ in 0..seen_milestones_num {
                    line.clear();
                    let hash = match reader.read_line(&mut line) {
                        Ok(_) => {
                            let tokens: Vec<&str> = line.split(";").collect();
                            // TODO check size of tokens
                            // TODO what to do with index ?
                            // TODO check trytes size
                            match TryteBuf::try_from_str(&tokens[0][..tokens[0].len()]) {
                                Ok(buf) => Hash::try_from_inner(buf.as_trits().encode::<T1B1Buf>())
                                    .map_err(|_| SnapshotMetadataError::InvalidSeenMilestoneHash)?,
                                Err(_) => Err(SnapshotMetadataError::InvalidSeenMilestoneHash)?,
                            }
                        }
                        Err(e) => Err(SnapshotMetadataError::IOError(e))?,
                    };
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

    pub fn hash(&self) -> &TritBuf<T1B1Buf> {
        &self.hash
    }

    pub fn index(&self) -> u64 {
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

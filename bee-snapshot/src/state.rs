use bee_ternary::{
    T1B1Buf,
    TritBuf,
    TryteBuf,
};

use std::fs::File;
use std::io::{
    BufRead,
    BufReader,
};

// TODO export ?
pub const IOTA_SUPPLY: u64 = 2779530283277761;

#[derive(Debug)]
pub enum SnapshotStateError {
    IOError(std::io::Error),
    InvalidHash,
    InvalidBalance(std::num::ParseIntError),
    InvalidSupply(u64, u64),
}

pub struct SnapshotState {
    entries: Vec<(TritBuf<T1B1Buf>, u64)>,
}

impl SnapshotState {
    pub fn new(path: &str) -> Result<Self, SnapshotStateError> {
        match File::open(path) {
            Ok(file) => {
                let reader = BufReader::new(file);
                let mut supply: u64 = 0;
                // TODO any possibility to reserve ?
                let mut entries = Vec::new();

                for line in reader.lines() {
                    match line {
                        Ok(line) => {
                            let tokens: Vec<&str> = line.split(";").collect();
                            // TODO check size of tokens
                            // TODO check trytes size

                            let hash = match TryteBuf::try_from_str(&tokens[0][..tokens[0].len()]) {
                                Ok(buf) => buf.as_trits().encode::<T1B1Buf>(),
                                Err(_) => Err(SnapshotStateError::InvalidHash)?,
                            };

                            let balance = tokens[1][..tokens[1].len()]
                                .parse::<u64>()
                                .map_err(|e| SnapshotStateError::InvalidBalance(e))?;

                            entries.push((hash, balance));

                            supply += balance;
                        }
                        Err(e) => Err(SnapshotStateError::IOError(e))?,
                    }
                }

                if supply != IOTA_SUPPLY {
                    Err(SnapshotStateError::InvalidSupply(supply, IOTA_SUPPLY))?;
                }

                Ok(Self { entries })
            }
            Err(e) => Err(SnapshotStateError::IOError(e)),
        }
    }

    pub fn entries(&self) -> &Vec<(TritBuf<T1B1Buf>, u64)> {
        &self.entries
    }
}

use bee_bundle::{Address, TransactionField};
use bee_ternary::{T1B1Buf, TryteBuf};

use std::{
    collections::HashMap,
    fs::File,
    io::{BufRead, BufReader},
};

// TODO export ?
pub const IOTA_SUPPLY: u64 = 2_779_530_283_277_761;

#[derive(Debug)]
pub enum SnapshotStateError {
    IOError(std::io::Error),
    InvalidAddress,
    InvalidBalance(std::num::ParseIntError),
    InvalidSupply(u64, u64),
}

pub struct SnapshotState {
    state: HashMap<Address, u64>,
}

impl SnapshotState {
    pub fn new(path: &str) -> Result<Self, SnapshotStateError> {
        match File::open(path) {
            Ok(file) => {
                let reader = BufReader::new(file);
                let mut supply: u64 = 0;
                // TODO any possibility to reserve ?
                let mut state = HashMap::new();

                for line in reader.lines() {
                    match line {
                        Ok(line) => {
                            let tokens: Vec<&str> = line.split(";").collect();
                            // TODO check size of tokens

                            let hash = match TryteBuf::try_from_str(&tokens[0][..tokens[0].len()]) {
                                Ok(buf) => Address::try_from_inner(buf.as_trits().encode::<T1B1Buf>())
                                    .map_err(|_| SnapshotStateError::InvalidAddress),
                                Err(_) => Err(SnapshotStateError::InvalidAddress),
                            }?;

                            let balance = tokens[1][..tokens[1].len()]
                                .parse::<u64>()
                                .map_err(|e| SnapshotStateError::InvalidBalance(e))?;

                            state.insert(hash, balance);

                            supply += balance;
                        }
                        Err(e) => return Err(SnapshotStateError::IOError(e)),
                    }
                }

                if supply != IOTA_SUPPLY {
                    return Err(SnapshotStateError::InvalidSupply(supply, IOTA_SUPPLY));
                }

                Ok(Self { state })
            }
            Err(e) => Err(SnapshotStateError::IOError(e)),
        }
    }

    pub fn state(&self) -> &HashMap<Address, u64> {
        &self.state
    }

    pub fn into_state(self) -> HashMap<Address, u64> {
        self.state
    }
}

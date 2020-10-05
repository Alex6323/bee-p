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

use crate::{constants::IOTA_SUPPLY, global::GlobalSnapshot};

use bee_ledger::state::LedgerState;
use bee_ternary::{T1B1Buf, TryteBuf};
use bee_transaction::bundled::{Address, BundledTransactionField};

use std::{
    fs::File,
    io::{prelude::*, BufReader},
};

#[derive(Eq, PartialEq, Debug)]
pub enum Error {
    FileNotFound,
    FailedIO,
    MissingSemicolon,
    ExtraneousSemicolon,
    InvalidAddressTryte,
    InvalidAddressLength,
    DuplicateAddress,
    InvalidBalance,
    NullBalance,
    InvalidSupply,
    DifferentNewline,
}

impl GlobalSnapshot {
    pub fn from_file(path: &str, index: u32) -> Result<Self, Error> {
        let file = File::open(path).map_err(|_| Error::FileNotFound)?;
        let reader = BufReader::new(file);

        let mut state = LedgerState::new();
        let mut supply = 0;

        for line in reader.lines() {
            let line = line.map_err(|_| Error::FailedIO)?;
            let tokens = line.split(";").collect::<Vec<&str>>();

            if tokens.len() < 2 {
                return Err(Error::MissingSemicolon);
            } else if tokens.len() > 2 {
                return Err(Error::ExtraneousSemicolon);
            }

            let address = match TryteBuf::try_from_str(tokens[0]) {
                Ok(trytes) => Address::try_from_inner(trytes.as_trits().encode::<T1B1Buf>())
                    .map_err(|_| Error::InvalidAddressLength)?,
                Err(_) => return Err(Error::InvalidAddressTryte),
            };

            let balance = tokens[1].parse::<u64>().map_err(|_| Error::InvalidBalance)?;

            if balance == 0 {
                return Err(Error::NullBalance);
            }

            if state.insert(address, balance).is_some() {
                return Err(Error::DuplicateAddress);
            }

            supply += balance;
        }

        if supply != IOTA_SUPPLY {
            return Err(Error::InvalidSupply);
        }

        Ok(Self { state, index })
    }
}

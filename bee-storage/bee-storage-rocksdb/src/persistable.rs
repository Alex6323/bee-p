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

use bee_storage::persistable::Persistable;

use crate::storage::Storage;

use bee_crypto::ternary::Hash;
// use bee_ledger::{diff::LedgerDiff, state::LedgerState};
use bee_protocol::{
    tangle::{flags::Flags, TransactionMetadata},
    MilestoneIndex,
};

use std::{collections::HashMap, convert::TryInto};

pub const LE_0_BYTES_LEN: [u8; 4] = [0, 0, 0, 0];

// Auto trait implementations;
impl Persistable<Storage> for u32 {
    fn write_to(&self, buffer: &mut Vec<u8>) {
        buffer.extend(&u32::to_le_bytes(*self));
    }
    fn read_from(slice: &[u8]) -> Self {
        Self::from_le_bytes(slice[..4].try_into().unwrap())
    }
}

impl Persistable<Storage> for i64 {
    fn write_to(&self, buffer: &mut Vec<u8>) {
        buffer.extend(&i64::to_le_bytes(*self));
    }
    fn read_from(slice: &[u8]) -> Self {
        Self::from_le_bytes(slice[..8].try_into().unwrap())
    }
}

impl Persistable<Storage> for u64 {
    fn write_to(&self, buffer: &mut Vec<u8>) {
        buffer.extend(&u64::to_le_bytes(*self));
    }
    fn read_from(slice: &[u8]) -> Self {
        Self::from_le_bytes(slice[..8].try_into().unwrap())
    }
}

impl Persistable<Storage> for u8 {
    fn write_to(&self, buffer: &mut Vec<u8>) {
        buffer.extend(&u8::to_le_bytes(*self));
    }
    fn read_from(slice: &[u8]) -> Self {
        Self::from_le_bytes(slice[..1].try_into().unwrap())
    }
}

impl<K, V, S: ::std::hash::BuildHasher + Default> Persistable<Storage> for HashMap<K, V, S>
where
    K: Eq + std::hash::Hash + Persistable<Storage>,
    V: Persistable<Storage>,
{
    fn write_to(&self, buffer: &mut Vec<u8>) {
        // extend key_value pairs count of the hashmap into the buffer
        buffer.extend(&i32::to_le_bytes(self.len() as i32));
        let mut current_k_or_v_position;
        let mut k_or_v_byte_size;
        // iter on hashmap pairs;
        for (k, v) in self {
            // extend k-0-length;
            buffer.extend(&LE_0_BYTES_LEN);
            current_k_or_v_position = buffer.len();
            // encode key into the buffer
            k.write_to(buffer);
            // calculate the actual byte_size of the key;
            k_or_v_byte_size = buffer.len() - current_k_or_v_position;
            // change the k-0-length to reflect the actual key length;
            buffer[(current_k_or_v_position - 4)..current_k_or_v_position]
                .copy_from_slice(&i32::to_le_bytes(k_or_v_byte_size as i32));
            // extend v-0-length;
            buffer.extend(&LE_0_BYTES_LEN);
            current_k_or_v_position = buffer.len();
            // encode value into the buffer
            v.write_to(buffer);
            // calculate the actual byte_size of the value;
            k_or_v_byte_size = buffer.len() - current_k_or_v_position;
            // change the k-0-length to reflect the actual value length;
            buffer[(current_k_or_v_position - 4)..current_k_or_v_position]
                .copy_from_slice(&i32::to_le_bytes(k_or_v_byte_size as i32));
        }
    }

    fn read_from(slice: &[u8]) -> Self {
        let mut length;
        let map_len = i32::from_le_bytes(slice[0..4].try_into().unwrap()) as usize;
        let mut map: HashMap<K, V, S> = HashMap::default();
        let mut pair_start = 4;
        for _ in 0..map_len {
            // decode key_byte_size
            let key_start = pair_start + 4;
            length = i32::from_le_bytes(slice[pair_start..key_start].try_into().unwrap()) as usize;
            // modify pair_start to be the vlength_start
            pair_start = key_start + length;
            let k = K::read_from(&slice[key_start..pair_start]);
            let value_start = pair_start + 4;
            length = i32::from_le_bytes(slice[pair_start..value_start].try_into().unwrap()) as usize;
            // next pair_start
            pair_start = value_start + length;
            let v = V::read_from(&slice[value_start..pair_start]);
            // insert key,value
            map.insert(k, v);
        }
        map
    }
}

impl Persistable<Storage> for TransactionMetadata {
    fn write_to(&self, buffer: &mut Vec<u8>) {
        // encode struct in order
        // 1- encode flags
        self.flags().bits().write_to(buffer);
        // 2- encode milestone_index
        self.milestone_index().write_to(buffer);
        // 3- encode arrival_timestamp
        self.arrival_timestamp().write_to(buffer);
        // 4- encode solidification_timestamp
        self.solidification_timestamp().write_to(buffer);
        // 5- encode confirmation_timestamp
        self.confirmation_timestamp().write_to(buffer);
        // 6- encode cone_index
        self.cone_index().write_to(buffer);
        // 7- encode otrsi
        self.otrsi().write_to(buffer);
        // 8- encode ytrsi
        self.ytrsi().write_to(buffer);
    }
    fn read_from(slice: &[u8]) -> Self {
        // decode struct in order
        // 1- decode flags
        let flags = Flags::from_bits(u8::read_from(&slice[0..1])).unwrap();
        // 2- decode milestone_index
        let milestone_index = MilestoneIndex::read_from(&slice[1..5]);
        // 3- decode arrival_timestamp
        let arrival_timestamp = u64::read_from(&slice[5..13]);
        // 4- decode solidification_timestamp
        let solidification_timestamp = u64::read_from(&slice[13..21]);
        // 5- decode confirmation_timestamp
        let confirmation_timestamp = u64::read_from(&slice[21..29]);
        // 6- decode cone_index
        let mut head = 29;
        let cone_index = Option::<MilestoneIndex>::read_from(&slice[head..]);
        if cone_index.is_some() {
            head += 6 // (6 is 2-bytes for short[n] and 4-bytes for u32)
        } else {
            head += 2 // (2 is 2-bytes for short[n] == -1)
        }
        // 7- decode otrsi
        let otrsi = Option::<MilestoneIndex>::read_from(&slice[head..]);
        if otrsi.is_some() {
            head += 6 // (6 is 2-bytes for short[n] and 4-bytes for u32)
        } else {
            head += 2 // (2 is 2-bytes for short[n] == -1)
        }
        // 7- decode ytrsi
        let ytrsi = Option::<MilestoneIndex>::read_from(&slice[head..]);
        Self::new(
            flags,
            milestone_index,
            arrival_timestamp,
            solidification_timestamp,
            confirmation_timestamp,
            cone_index,
            otrsi,
            ytrsi,
        )
    }
}

// impl Persistable<Storage> for LedgerDiff {
//     fn write_to(&self, buffer: &mut Vec<u8>) {
//         self.inner().write_to(buffer)
//     }
//     fn read_from(slice: &[u8]) -> Self {
//         LedgerDiff::from(HashMap::read_from(slice))
//     }
// }
//
// impl Persistable<Storage> for LedgerState {
//     fn write_to(&self, buffer: &mut Vec<u8>) {
//         self.inner().write_to(buffer)
//     }
//     fn read_from(slice: &[u8]) -> Self {
//         Self::from(HashMap::read_from(slice))
//     }
// }

impl Persistable<Storage> for MilestoneIndex {
    fn write_to(&self, buffer: &mut Vec<u8>) {
        self.0.write_to(buffer)
    }
    fn read_from(slice: &[u8]) -> Self {
        MilestoneIndex(u32::read_from(slice))
    }
}

impl Persistable<Storage> for Option<MilestoneIndex> {
    fn write_to(&self, buffer: &mut Vec<u8>) {
        if let Some(ms_index) = self {
            // encode short[n] where 4 == size_of::<u32>();
            buffer.extend(&i16::to_le_bytes(4));
            // encode the milestone_index
            ms_index.write_to(buffer);
        } else {
            // encode short[n] where n = -1 == null == None;
            buffer.extend(&i16::to_le_bytes(-1));
        }
    }
    fn read_from(slice: &[u8]) -> Self {
        // decode short[n]
        let length = i16::from_le_bytes(slice[0..2].try_into().unwrap());
        if length < 0 {
            None
        } else {
            // decode starting from the slice[2..]
            Some(MilestoneIndex(u32::read_from(&slice[2..])))
        }
    }
}

impl Persistable<Storage> for Hash {
    fn write_to(&self, _buffer: &mut Vec<u8>) {
        todo!()
    }
    fn read_from(_slice: &[u8]) -> Self {
        todo!()
    }
}

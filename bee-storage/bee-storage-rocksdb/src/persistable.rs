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
use bee_ledger::{diff::LedgerDiff, state::LedgerState};
use bee_protocol::{
    tangle::{flags::Flags, TransactionMetadata},
    MilestoneIndex,
};
use bee_transaction::bundled::{Address, BundledTransaction};

use std::{collections::HashMap, convert::TryInto};

pub const LE_0_BYTES_LEN: [u8; 4] = [0, 0, 0, 0];

// Auto trait implementations;
impl Persistable<Storage> for u32 {
    fn encode_persistable<Storage>(&self, buffer: &mut Vec<u8>) {
        buffer.extend(&u32::to_le_bytes(*self));
    }
    fn decode_persistable<Storage>(slice: &[u8]) -> Self {
        Self::from_le_bytes(slice.try_into().unwrap())
    }
}

impl Persistable<Storage> for i64 {
    fn encode_persistable<Storage>(&self, buffer: &mut Vec<u8>) {
        buffer.extend(&i64::to_le_bytes(*self));
    }
    fn decode_persistable<Storage>(slice: &[u8]) -> Self {
        i64::from_le_bytes(slice.try_into().unwrap())
    }
}

impl Persistable<Storage> for u64 {
    fn encode_persistable<Storage>(&self, buffer: &mut Vec<u8>) {
        buffer.extend(&u64::to_le_bytes(*self));
    }
    fn decode_persistable<Storage>(slice: &[u8]) -> Self {
        u64::from_le_bytes(slice.try_into().unwrap())
    }
}

impl Persistable<Storage> for u8 {
    fn encode_persistable<Storage>(&self, buffer: &mut Vec<u8>) {
        buffer.extend(&u8::to_le_bytes(*self));
    }
    fn decode_persistable<Storage>(slice: &[u8]) -> Self {
        u8::from_le_bytes(slice.try_into().unwrap())
    }
}

impl<K, V, S: ::std::hash::BuildHasher + Default> Persistable<Storage> for HashMap<K, V, S>
where
    K: Eq + std::hash::Hash + Persistable<Storage>,
    V: Persistable<Storage>,
{
    fn encode_persistable<Storage>(&self, buffer: &mut Vec<u8>) {
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
            k.encode_persistable::<Storage>(buffer);
            // calculate the actual byte_size of the key;
            k_or_v_byte_size = buffer.len() - current_k_or_v_position;
            // change the k-0-length to reflect the actual key length;
            buffer[(current_k_or_v_position - 4)..current_k_or_v_position]
                .copy_from_slice(&i32::to_le_bytes(k_or_v_byte_size as i32));
            // extend v-0-length;
            buffer.extend(&LE_0_BYTES_LEN);
            current_k_or_v_position = buffer.len();
            // encode value into the buffer
            v.encode_persistable::<Storage>(buffer);
            // calculate the actual byte_size of the value;
            k_or_v_byte_size = buffer.len() - current_k_or_v_position;
            // change the k-0-length to reflect the actual value length;
            buffer[(current_k_or_v_position - 4)..current_k_or_v_position]
                .copy_from_slice(&i32::to_le_bytes(k_or_v_byte_size as i32));
        }
    }

    fn decode_persistable<Storage>(slice: &[u8]) -> Self {
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
            let k = K::decode_persistable::<Storage>(&slice[key_start..pair_start]);
            let value_start = pair_start + 4;
            length = i32::from_le_bytes(slice[pair_start..value_start].try_into().unwrap()) as usize;
            // next pair_start
            pair_start = value_start + length;
            let v = V::decode_persistable::<Storage>(&slice[value_start..pair_start]);
            // insert key,value
            map.insert(k, v);
        }
        map
    }
}

impl Persistable<Storage> for TransactionMetadata {
    fn encode_persistable<Storage>(&self, buffer: &mut Vec<u8>) {
        // encode struct in order
        // 1- encode flags
        self.flags().bits().encode_persistable::<Storage>(buffer);
        // 2- encode milestone_index
        self.milestone_index().encode_persistable::<Storage>(buffer);
        // 3- encode arrival_timestamp
        self.arrival_timestamp().encode_persistable::<Storage>(buffer);
        // 4- encode solidification_timestamp
        self.solidification_timestamp().encode_persistable::<Storage>(buffer);
        // 5- encode confirmation_timestamp
        self.confirmation_timestamp().encode_persistable::<Storage>(buffer);
    }
    fn decode_persistable<Storage>(slice: &[u8]) -> Self {
        // decode struct in order
        // 1- decode flags
        let flags = Flags::from_bits(u8::decode_persistable::<Storage>(&slice[0..1])).unwrap();
        // 2- decode milestone_index
        let milestone_index = MilestoneIndex::decode_persistable::<Storage>(&slice[1..5]);
        // 3- decode arrival_timestamp
        let arrival_timestamp = u64::decode_persistable::<Storage>(&slice[5..13]);
        // 4- decode solidification_timestamp
        let solidification_timestamp = u64::decode_persistable::<Storage>(&slice[13..21]);
        // 5- decode confirmation_timestamp
        let confirmation_timestamp = u64::decode_persistable::<Storage>(&slice[21..29]);

        Self::new(
            flags,
            milestone_index,
            arrival_timestamp,
            solidification_timestamp,
            confirmation_timestamp,
        )
    }
}

impl Persistable<Storage> for LedgerDiff {
    fn encode_persistable<Storage>(&self, buffer: &mut Vec<u8>) {
        self.inner().encode_persistable::<Storage>(buffer)
    }
    fn decode_persistable<Storage>(slice: &[u8]) -> Self {
        LedgerDiff::from(HashMap::decode_persistable::<Storage>(slice))
    }
}

impl Persistable<Storage> for LedgerState {
    fn encode_persistable<Storage>(&self, buffer: &mut Vec<u8>) {
        self.inner().encode_persistable::<Storage>(buffer)
    }
    fn decode_persistable<Storage>(slice: &[u8]) -> Self {
        Self::from(HashMap::decode_persistable::<Storage>(slice))
    }
}

impl Persistable<Storage> for MilestoneIndex {
    fn encode_persistable<Storage>(&self, buffer: &mut Vec<u8>) {
        self.0.encode_persistable::<Storage>(buffer)
    }
    fn decode_persistable<Storage>(slice: &[u8]) -> Self {
        MilestoneIndex(u32::decode_persistable::<Storage>(slice))
    }
}

impl Persistable<Storage> for Address {
    fn encode_persistable<Storage>(&self, _buffer: &mut Vec<u8>) {
        todo!()
    }
    fn decode_persistable<Storage>(_slice: &[u8]) -> Self {
        todo!()
    }
}

impl Persistable<Storage> for Hash {
    fn encode_persistable<Storage>(&self, _buffer: &mut Vec<u8>) {
        todo!()
    }
    fn decode_persistable<Storage>(_slice: &[u8]) -> Self {
        todo!()
    }
}

impl Persistable<Storage> for BundledTransaction {
    fn encode_persistable<Storage>(&self, _buffer: &mut Vec<u8>) {
        todo!()
    }
    fn decode_persistable<Storage>(_slice: &[u8]) -> Self {
        todo!()
    }
}

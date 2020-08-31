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
pub mod constants;

use std::{collections::HashMap, convert::TryInto, hash::Hash};

pub trait RocksDBPersistable {
    /// This encode method will extend the provided buffer and return ();
    fn encode_persistable(&self, buffer: &mut Vec<u8>);
    /// Decode `slice` and return Self
    fn decode_persistable(slice: &[u8]) -> Self
    where
        Self: Sized;
}

#[cfg(feature = "rocks_db")]
#[allow(unused_imports)]
pub use RocksDBPersistable as Persistable;

// Auto implementations;

impl RocksDBPersistable for i64 {
    fn encode_persistable(&self, buffer: &mut Vec<u8>) {
        buffer.extend(&i64::to_le_bytes(*self));
    }
    fn decode_persistable(slice: &[u8]) -> Self {
        i64::from_le_bytes(slice.try_into().unwrap())
    }
}

impl<K, V, S: ::std::hash::BuildHasher + Default> RocksDBPersistable for HashMap<K, V, S>
where
    K: Eq + Hash + RocksDBPersistable,
    V: RocksDBPersistable,
{
    fn encode_persistable(&self, buffer: &mut Vec<u8>) {
        // extend key_value pairs count of the hashmap into the buffer
        buffer.extend(&i32::to_le_bytes(self.len() as i32));
        let mut current_k_or_v_position;
        let mut k_or_v_byte_size;
        // iter on hashmap pairs;
        for (k, v) in self {
            // extend k-0-length;
            buffer.extend(&constants::LE_0_BYTES_LEN);
            current_k_or_v_position = buffer.len();
            // encode key into the buffer
            k.encode_persistable(buffer);
            // calculate the actual byte_size of the key;
            k_or_v_byte_size = buffer.len() - current_k_or_v_position;
            // change the k-0-length to reflect the actual key length;
            buffer[(current_k_or_v_position - 4)..current_k_or_v_position]
                .copy_from_slice(&i32::to_le_bytes(k_or_v_byte_size as i32));
            // extend v-0-length;
            buffer.extend(&constants::LE_0_BYTES_LEN);
            current_k_or_v_position = buffer.len();
            // encode value into the buffer
            v.encode_persistable(buffer);
            // calculate the actual byte_size of the value;
            k_or_v_byte_size = buffer.len() - current_k_or_v_position;
            // change the k-0-length to reflect the actual value length;
            buffer[(current_k_or_v_position - 4)..current_k_or_v_position]
                .copy_from_slice(&i32::to_le_bytes(k_or_v_byte_size as i32));
        }
    }

    fn decode_persistable(slice: &[u8]) -> Self {
        let mut length;
        let map_len = i32::from_le_bytes(slice[0..4].try_into().unwrap()) as usize;
        let mut map: HashMap<K, V, S> = HashMap::default();
        let mut pair_start = 4;
        for _ in 0..map_len {
            // decode key_byte_size
            let key_start = pair_start + 4;
            length = i32::from_le_bytes(slice[pair_start..key_start].try_into().unwrap()) as usize;
            let k = K::decode_persistable(&slice[key_start..(key_start + length)]);
            // modify pair_start to be the vlength_start
            pair_start = key_start + length;
            let value_start = pair_start + 4;
            length = i32::from_le_bytes(slice[pair_start..value_start].try_into().unwrap()) as usize;
            let v = V::decode_persistable(&slice[value_start..(value_start + length)]);
            // insert key,value
            map.insert(k, v);
            // next pair_start
            pair_start = value_start + length;
        }
        map
    }
}

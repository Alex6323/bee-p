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

// use crate::packet::{v0::MESSAGES_VERSION_0, v2::MESSAGES_VERSION_2};

pub(crate) const PACKETS_VERSIONS: [u8; 16] = [0, 0, 0, 0, 0, 0, 0, 64, 0, 0, 0, 0, 0, 0, 0, 4];
// pub(crate) const PACKETS_VERSIONS: [u8; 1] = [MESSAGES_VERSION_0 | MESSAGES_VERSION_2];

fn common_supported_version(own_supported_versions: &[u8], supported_versions: &[u8]) -> Result<u8, u8> {
    let mut highest_supported_version: u8 = 0;

    for (i, own_supported_version) in own_supported_versions.iter().enumerate() {
        if i >= supported_versions.len() {
            break;
        }

        let supported: u8 = supported_versions[i] & own_supported_version;

        if supported == 0 {
            continue;
        }

        let mut highest: u8 = 0;
        for j in 0..8 {
            if ((supported >> j) & 1) == 1 {
                highest = j + 1;
            }
        }

        highest_supported_version = highest + ((i as u8) * 8);
    }

    if highest_supported_version == 0 {
        // TODO remove unwrap by checking size of versions
        let last_versions_byte: &u8 = supported_versions.last().unwrap();
        let mut highest: u8 = 0;

        for j in 0..8 {
            if ((last_versions_byte >> j) & 1) == 1 {
                highest = j + 1;
            }
        }

        return Err(highest + ((supported_versions.len() - 1) as u8 * 8));
    }

    Ok(highest_supported_version)
}

pub(crate) fn packets_supported_version(supported_versions: &[u8]) -> Result<u8, u8> {
    common_supported_version(&PACKETS_VERSIONS, supported_versions)
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test() {
        for i in 0..32 {
            let mut byte = 1;

            for j in 0..8 {
                let mut version = [0; 32];
                let expected_version = i as u16 * 8 + j as u16 + 1;

                version[i] = byte;
                if expected_version != 256 {
                    match common_supported_version(&version, &version) {
                        Ok(v) => assert_eq!(v, expected_version as u8),
                        _ => unreachable!(),
                    }
                }
                byte <<= 1;
            }
        }
    }
}

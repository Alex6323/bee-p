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

use crate::manual::{ManualPeeringConfig, ManualPeeringConfigBuilder};

use bee_network::Keypair;

use serde::Deserialize;

#[derive(Default, Deserialize)]
pub struct PeeringConfigBuilder {
    local_keypair: Option<String>,
    manual: ManualPeeringConfigBuilder,
}

impl PeeringConfigBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn local_keypair(mut self, local_keypair: String) -> Self {
        self.local_keypair.replace(local_keypair);
        self
    }

    pub fn finish(self) -> PeeringConfig {
        let (keypair, kp_string, new) = if let Some(kp_string) = self.local_keypair {
            if kp_string.len() == 128 {
                let mut decoded = [0u8; 64];
                hex::decode_to_slice(&kp_string[..], &mut decoded).expect("error decoding local keypair");
                let keypair = Keypair::decode(&mut decoded).expect("error decoding local keypair");
                (keypair, kp_string, false)
            } else if kp_string.is_empty() {
                generate_random_local_keypair()
            } else {
                panic!("invalid keypair string length");
            }
        } else {
            generate_random_local_keypair()
        };

        PeeringConfig {
            local_keypair: (keypair, kp_string, new),
            manual: self.manual.finish(),
        }
    }
}

fn generate_random_local_keypair() -> (Keypair, String, bool) {
    let keypair = Keypair::generate();
    let encoded = keypair.encode();
    let kp_text = hex::encode(encoded);
    (keypair, kp_text, true)
}

#[derive(Clone)]
pub struct PeeringConfig {
    pub local_keypair: (Keypair, String, bool),
    pub manual: ManualPeeringConfig,
}

impl PeeringConfig {
    pub fn build() -> PeeringConfigBuilder {
        PeeringConfigBuilder::new()
    }
}

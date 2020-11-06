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
        let local_keypair = if let Some(local_keypair) = self.local_keypair {
            if local_keypair.len() != 128 {
                panic!("invalid keypair length");
            }
            let mut decoded = [0u8; 64];
            hex::decode_to_slice(local_keypair, &mut decoded).expect("error decoding local keypair");
            Keypair::decode(&mut decoded).expect("error decoding local keypair")
        } else {
            let keypair = Keypair::generate();
            let encoded = keypair.encode();
            let s = hex::encode(encoded);
            log::info!("Generated new local keypair: {}", s);
            keypair
        };

        PeeringConfig {
            local_keypair,
            manual: self.manual.finish(),
        }
    }
}

#[derive(Clone)]
pub struct PeeringConfig {
    pub local_keypair: Keypair,
    pub manual: ManualPeeringConfig,
}

impl PeeringConfig {
    pub fn build() -> PeeringConfigBuilder {
        PeeringConfigBuilder::new()
    }
}

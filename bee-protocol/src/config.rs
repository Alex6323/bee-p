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

use serde::Deserialize;

const DEFAULT_MWM: u8 = 14;
// TODO change
const DEFAULT_COO_PUBLIC_KEY: &str =
    "52fdfc072182654f163f5f0f9a621d729566c74d10037c4d7bbb0407d1e2c649";
const DEFAULT_TRANSACTION_WORKER_CACHE: usize = 10000;
const DEFAULT_STATUS_INTERVAL: u64 = 10;
const DEFAULT_HANDSHAKE_WINDOW: u64 = 10;
const DEFAULT_MS_SYNC_COUNT: u32 = 1;

#[derive(Default, Deserialize)]
struct ProtocolCoordinatorConfigBuilder {
    public_key: Option<String>,
}

#[derive(Default, Deserialize)]
struct ProtocolWorkersConfigBuilder {
    transaction_worker_cache: Option<usize>,
    status_interval: Option<u64>,
    ms_sync_count: Option<u32>,
}

#[derive(Default, Deserialize)]
pub struct ProtocolConfigBuilder {
    mwm: Option<u8>,
    coordinator: ProtocolCoordinatorConfigBuilder,
    workers: ProtocolWorkersConfigBuilder,
    handshake_window: Option<u64>,
}

impl ProtocolConfigBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn mwm(mut self, mwm: u8) -> Self {
        self.mwm.replace(mwm);
        self
    }

    pub fn coo_public_key(mut self, coo_public_key: String) -> Self {
        self.coordinator.public_key.replace(coo_public_key);
        self
    }

    pub fn transaction_worker_cache(mut self, transaction_worker_cache: usize) -> Self {
        self.workers.transaction_worker_cache.replace(transaction_worker_cache);
        self
    }

    pub fn ms_sync_count(mut self, ms_sync_count: u32) -> Self {
        self.workers.ms_sync_count.replace(ms_sync_count);
        self
    }

    pub fn status_interval(mut self, status_interval: u64) -> Self {
        self.workers.status_interval.replace(status_interval);
        self
    }

    pub fn handshake_window(mut self, handshake_window: u64) -> Self {
        self.handshake_window.replace(handshake_window);
        self
    }

    pub fn finish(self) -> ProtocolConfig {
        // TODO handle unwrap by having default value
        let coo_public_key = hex::decode(
            &self
                .coordinator
                .public_key
                .unwrap_or_else(|| DEFAULT_COO_PUBLIC_KEY.to_owned()),
        )
        .unwrap();

        // TODO check length of public_key against 32

        let mut public_key_bytes = [0u8; 32];
        public_key_bytes.copy_from_slice(&coo_public_key);

        ProtocolConfig {
            mwm: self.mwm.unwrap_or(DEFAULT_MWM),
            coordinator: ProtocolCoordinatorConfig {
                public_key: public_key_bytes,
            },
            workers: ProtocolWorkersConfig {
                transaction_worker_cache: self
                    .workers
                    .transaction_worker_cache
                    .unwrap_or(DEFAULT_TRANSACTION_WORKER_CACHE),
                status_interval: self.workers.status_interval.unwrap_or(DEFAULT_STATUS_INTERVAL),
                ms_sync_count: self.workers.ms_sync_count.unwrap_or(DEFAULT_MS_SYNC_COUNT),
            },
            handshake_window: self.handshake_window.unwrap_or(DEFAULT_HANDSHAKE_WINDOW),
        }
    }
}

#[derive(Clone)]
pub struct ProtocolCoordinatorConfig {
    // TODO real PK type ?
    pub(crate) public_key: [u8; 32],
}

#[derive(Clone)]
pub struct ProtocolWorkersConfig {
    pub(crate) transaction_worker_cache: usize,
    pub(crate) status_interval: u64,
    pub(crate) ms_sync_count: u32,
}

#[derive(Clone)]
pub struct ProtocolConfig {
    pub(crate) mwm: u8,
    pub(crate) coordinator: ProtocolCoordinatorConfig,
    pub(crate) workers: ProtocolWorkersConfig,
    pub(crate) handshake_window: u64,
}

impl ProtocolConfig {
    pub fn build() -> ProtocolConfigBuilder {
        ProtocolConfigBuilder::new()
    }

    pub fn coordinator(&self) -> &ProtocolCoordinatorConfig {
        &self.coordinator
    }
}

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

//! A crate that contains foundational building blocks for the IOTA Tangle.

mod config;
pub mod rocksdb;
use async_trait::async_trait;
use std::{error::Error, fs};
#[async_trait]
/// Trait to be implemented on storage backend,
/// which determine how to start and shutdown the storage
pub trait Backend {
    /// start method should impl how to start and initialize the corrsponding database
    /// It takes config_path which define the database options, and returns Result<Self, Box<dyn Error>>
    async fn start(config_path: String) -> Result<Self, Box<dyn Error>> where Self: Sized;
    /// shutdown method should impl how to shutdown the corrsponding database
    /// It takes the ownership of self, and returns () or error
    async fn shutdown(self) -> Result<(), Box<dyn Error>>;
}
// rocksdb storage support as backend
#[cfg(feature = "rocks_db")]
/// RocksDB Storage struct
pub struct Storage {
    pub inner: ::rocksdb::DB,
}

#[cfg(feature = "rocks_db")]
#[async_trait]
impl Backend for Storage {
    /// It starts RocksDB instance and then initialize the required column familes
    async fn start(config_path: String) -> Result<Self, Box<dyn Error>> {
        let config_as_string = fs::read_to_string(config_path)?;
        let config: config::Config = toml::from_str(&config_as_string)?;
        let db = rocksdb::RocksdbBackend::new(config)?;
        Ok(Storage { inner: db })
    }
    /// It shutdown RocksDB instance,
    /// Note: the shutdown is done through flush method and then droping the storage object
    async fn shutdown(self) -> Result<(), Box<dyn Error>> {
        if let Err(e) = self.inner.flush() {
            return Err(Box::new(e))
        }
        Ok(())
    }
}

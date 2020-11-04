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

use async_trait::async_trait;
use serde::de::DeserializeOwned;

use std::error::Error;

#[async_trait]
/// Trait to be implemented on storage backend, which determine how to start and shutdown the storage.
pub trait Backend: Sized + Send + Sync + 'static {
    type ConfigBuilder: Default + DeserializeOwned + Into<Self::Config>;
    type Config: Clone + Send;
    type Error: std::error::Error;
    /// Start method should impl how to start and initialize the corrsponding database.
    /// It takes config_path which define the database options, and returns Result<Self, Box<dyn Error>>.
    async fn start(config: Self::Config) -> Result<Self, Box<dyn Error>>;

    /// Shutdown method should impl how to shutdown the corrsponding database.
    /// It takes the ownership of self, and returns () or error.
    async fn shutdown(self) -> Result<(), Box<dyn Error>>;
}

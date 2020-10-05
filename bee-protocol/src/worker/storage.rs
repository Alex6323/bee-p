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
use bee_common_ext::{node::Node, worker::Worker};
use bee_storage::storage::Backend;
use std::{error, fmt};

#[derive(Debug)]
pub struct Error(Box<dyn error::Error>);

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl error::Error for Error {}

pub struct StorageWorker;

#[async_trait]
impl<N: Node> Worker<N> for StorageWorker {
    type Config = <N::Backend as Backend>::Config;
    type Error = Error;

    async fn start(node: &mut N, config: Self::Config) -> Result<Self, Self::Error> {
        let backend = N::Backend::start(config).await.map_err(Error)?;

        node.register_resource(backend);

        Ok(Self)
    }
}

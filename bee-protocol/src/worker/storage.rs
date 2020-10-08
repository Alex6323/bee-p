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

use bee_common_ext::{node::Node, worker::Worker};
use bee_storage::storage::Backend;

use async_trait::async_trait;
use log::{error, warn};
use tokio::time::interval;

use std::{
    error, fmt,
    time::{Duration, Instant},
};

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

    async fn stop(self, node: &mut N) -> Result<(), Self::Error> {
        let backend = if let Some(backend) = node.remove_resource::<N::Backend>() {
            backend
        } else {
            warn!(
                "The storage was still in use by other users when the storage worker stopped. \
                This is a bug, but not a critical one. From here, we'll revert to polling the \
                storage until other users are finished with it."
            );

            let poll_start = Instant::now();
            let poll_freq = 20;
            let mut interval = interval(Duration::from_millis(poll_freq));
            loop {
                match node.remove_resource::<N::Backend>() {
                    Some(backend) => break backend,
                    None => {
                        if Instant::now().duration_since(poll_start) > Duration::from_secs(5) {
                            error!(
                                "Storage shutdown polling period elapsed. The storage will be dropped \
                            without proper shutdown. This should be considered a bug."
                            );
                            return Ok(());
                        } else {
                            interval.tick().await;
                        }
                    }
                }
            }
        };

        backend.shutdown().await.map_err(Error)
    }
}

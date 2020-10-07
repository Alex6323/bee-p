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

use crate::worker::Worker;

use async_trait::async_trait;
use bee_common::shutdown;
use bee_storage::storage::Backend;
use futures::{channel::oneshot, future::Future};

use std::{any::Any, sync::Arc};

#[async_trait]
pub trait Node: Send + Sized + 'static {
    type Builder: NodeBuilder<Self>;
    type Backend: Backend;

    fn build() -> Self::Builder {
        Self::Builder::default()
    }

    async fn stop(mut self) -> Result<(), shutdown::Error>
    where
        Self: Sized;

    fn spawn<W, G, F>(&mut self, g: G)
    where
        Self: Sized,
        W: Worker<Self>,
        G: FnOnce(oneshot::Receiver<()>) -> F,
        F: Future<Output = ()> + Send + 'static;

    fn worker<W>(&self) -> Option<&W>
    where
        Self: Sized,
        W: Worker<Self> + Send + Sync;

    fn register_resource<R: Any + Send + Sync>(&mut self, res: R);

    fn remove_resource<R: Any + Send + Sync>(&mut self) -> Option<R>;

    fn resource<R: Any + Send + Sync>(&self) -> &Arc<R>;

    fn storage(&self) -> &Arc<Self::Backend> {
        self.resource()
    }
}

#[async_trait(?Send)]
pub trait NodeBuilder<N: Node>: Default {
    fn with_worker<W: Worker<N> + 'static>(self) -> Self
    where
        W::Config: Default;

    fn with_worker_cfg<W: Worker<N> + 'static>(self, config: W::Config) -> Self;

    async fn finish(self) -> N;
}

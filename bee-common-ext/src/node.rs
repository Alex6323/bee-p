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

use bee_common::shutdown;
use bee_storage::storage::Backend;

use async_trait::async_trait;
use futures::{channel::oneshot, future::Future};
use log::warn;

use std::{
    any::{type_name, Any},
    collections::HashMap,
    ops::Deref,
    panic::Location,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc, Mutex,
    },
};

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

    #[track_caller]
    fn resource<R: Any + Send + Sync>(&self) -> ResHandle<R>;

    fn storage(&self) -> ResHandle<Self::Backend> {
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

static RES_ID: AtomicUsize = AtomicUsize::new(0);

pub struct ResHandle<R> {
    id: Option<usize>,
    inner: Arc<(R, Mutex<HashMap<usize, &'static Location<'static>>>)>,
}

impl<R> ResHandle<R> {
    pub fn new(res: R) -> Self {
        Self {
            id: None,
            inner: Arc::new((res, Mutex::new(HashMap::new()))),
        }
    }

    pub fn try_unwrap(self) -> Option<R>
    where
        R: Any,
    {
        match Arc::try_unwrap(self.inner.clone()) {
            Ok((res, _)) => Some(res),
            Err(inner) => {
                let usages = inner
                    .1
                    .lock()
                    .unwrap()
                    .values()
                    .fold(String::new(), |s, loc| format!("{}\n- {}", s, loc));
                warn!(
                    "Attempted to gain ownership resource `{}` but it is still being used in the following places: {}",
                    type_name::<R>(),
                    usages,
                );
                None
            }
        }
    }
}

impl<R> Clone for ResHandle<R> {
    #[track_caller]
    fn clone(&self) -> Self {
        let new_id = RES_ID.fetch_add(1, Ordering::Relaxed);
        self.inner.1.lock().unwrap().insert(new_id, Location::caller());
        Self {
            id: Some(new_id),
            inner: self.inner.clone(),
        }
    }
}

impl<R> Deref for ResHandle<R> {
    type Target = R;

    fn deref(&self) -> &Self::Target {
        &self.inner.0
    }
}

impl<R> Drop for ResHandle<R> {
    fn drop(&mut self) {
        if let Some(id) = self.id {
            self.inner.1.lock().unwrap().remove(&id);
        }
    }
}

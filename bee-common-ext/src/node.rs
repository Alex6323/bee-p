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

use futures::{channel::oneshot, future::Future};
use log::info;
use bee_storage::storage::Backend;

use std::{
    any::{Any, TypeId, type_name},
    collections::{HashMap, HashSet},
    pin::Pin,
    sync::Arc,
};

pub trait Node: Default + Send + Sync + 'static {
    type Backend: Backend;

    fn new() -> Self;

    fn spawn<W, G, F>(&self, g: G)
    where
        Self: Sized,
        W: Worker<Self>,
        G: FnOnce(oneshot::Receiver<()>) -> F,
        F: Future<Output = ()> + Send + Sync + 'static;

    fn worker<W>(&self) -> Option<&W>
    where
        Self: Sized,
        W: Worker<Self> + Send + Sync;

    fn add_worker<W>(&mut self, worker: W)
    where
        Self: Sized,
        W: Worker<Self> + Send + Sync;

    fn register_resource<R: Any + Send + Sync>(&mut self, res: R);

    fn resource<R: Any + Send + Sync>(&self) -> &Arc<R>;

    fn storage(&self) -> &Arc<Self::Backend> { self.resource() }
}

type Maker<N> = dyn for<'a> FnOnce(&'a mut N) -> Pin<Box<dyn Future<Output = ()> + 'a>>;

#[derive(Default)]
pub struct NodeBuilder<N: Node> {
    deps: HashMap<TypeId, &'static [TypeId]>,
    makers: HashMap<TypeId, Box<Maker<N>>>,
}

impl<N: Node + 'static> NodeBuilder<N> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_worker<W: Worker<N> + 'static>(self) -> Self
    where
        W::Config: Default,
    {
        self.with_worker_cfg::<W>(W::Config::default())
    }

    pub fn with_worker_cfg<W: Worker<N> + 'static>(mut self, config: W::Config) -> Self {
        self.deps.insert(TypeId::of::<W>(), W::dependencies());
        self.makers.insert(
            TypeId::of::<W>(),
            Box::new(|node| {
                Box::pin(async move {
                    info!("Initializing worker `{}`...", type_name::<W>());
                    match W::start(node, config).await {
                        Ok(w) => node.add_worker(w),
                        Err(e) => panic!("Worker {} failed to start: {:?}.", type_name::<W>(), e),
                    }
                })
            }),
        );
        self
    }

    pub async fn finish(mut self) -> N {
        let mut node = N::new();

        for id in TopologicalOrder::sort(self.deps) {
            self.makers.remove(&id).unwrap()(&mut node).await;
        }

        node
    }
}

struct TopologicalOrder {
    graph: HashMap<TypeId, &'static [TypeId]>,
    non_visited: HashSet<TypeId>,
    being_visited: HashSet<TypeId>,
    order: Vec<TypeId>,
}

impl TopologicalOrder {
    fn visit(&mut self, id: TypeId) {
        if !self.non_visited.contains(&id) {
            return;
        }

        if !self.being_visited.insert(id) {
            panic!("Cyclic dependency detected.");
        }

        for &id in self.graph[&id] {
            self.visit(id);
        }

        self.being_visited.remove(&id);
        self.non_visited.remove(&id);
        self.order.push(id);
    }

    fn sort(graph: HashMap<TypeId, &'static [TypeId]>) -> Vec<TypeId> {
        let non_visited = graph.keys().copied().collect();

        let mut this = Self {
            graph,
            non_visited,
            being_visited: HashSet::new(),
            order: vec![],
        };

        while let Some(&id) = this.non_visited.iter().next() {
            this.visit(id);
        }

        this.order
    }
}

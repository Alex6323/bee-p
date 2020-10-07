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

use bee_common::shutdown;
use bee_common_ext::{
    node::{Node, NodeBuilder},
    worker::Worker,
};
use bee_storage::storage::Backend;

use anymap::{any::Any as AnyMapAny, Map};
use async_trait::async_trait;
use futures::{channel::oneshot, future::Future};
use log::info;
use tokio::spawn;

use std::{
    any::{type_name, Any, TypeId},
    collections::{HashMap, HashSet},
    marker::PhantomData,
    pin::Pin,
    sync::Arc,
};

type WorkerStart<N> = dyn for<'a> FnOnce(&'a mut N) -> Pin<Box<dyn Future<Output = ()> + 'a>>;
type WorkerStop<N> = dyn for<'a> FnOnce(&'a mut N) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> + Send;

#[allow(clippy::type_complexity)]
pub struct BeeNode<B> {
    workers: Map<dyn AnyMapAny + Send + Sync>,
    tasks: HashMap<
        TypeId,
        Vec<(
            oneshot::Sender<()>,
            // TODO Result ?
            Box<dyn Future<Output = Result<(), tokio::task::JoinError>> + Send + Sync + Unpin>,
        )>,
    >,
    resources: Map<dyn AnyMapAny + Send + Sync>,
    worker_stops: HashMap<TypeId, Box<WorkerStop<Self>>>,
    worker_order: Vec<TypeId>,
    phantom: PhantomData<B>,
}

impl<B: Backend> BeeNode<B> {
    fn add_worker<W: Worker<Self> + Send + Sync>(&mut self, worker: W) {
        self.workers.insert(worker);
    }

    fn remove_worker<W: Worker<Self> + Send + Sync>(&mut self) -> W {
        self.workers
            .remove()
            .unwrap_or_else(|| panic!("Failed to remove worker `{}`", type_name::<W>()))
    }
}

#[async_trait]
impl<B: Backend> Node for BeeNode<B> {
    type Builder = BeeNodeBuilder<B>;
    type Backend = B;

    async fn stop(mut self) -> Result<(), shutdown::Error>
    where
        Self: Sized,
    {
        for id in self.worker_order.clone().into_iter().rev() {
            self.worker_stops.remove(&id).unwrap()(&mut self).await;
            for (shutdown, task_fut) in self.tasks.remove(&id).unwrap_or_default() {
                let _ = shutdown.send(());
                // TODO: Should we handle thie error?
                let _ = task_fut.await; //.map_err(|e| shutdown::Error::from(worker::Error(Box::new(e))))?;
            }
        }

        Ok(())
    }

    fn register_resource<R: Any + Send + Sync>(&mut self, res: R) {
        self.resources.insert(Arc::new(res));
    }

    fn remove_resource<R: Any + Send + Sync>(&mut self) -> Option<R> {
        Arc::try_unwrap(self.resources.remove()?).ok()
    }

    #[track_caller]
    fn resource<R: Any + Send + Sync>(&self) -> &Arc<R> {
        self.resources
            .get()
            .unwrap_or_else(|| panic!("Unable to fetch node resource {}", type_name::<R>()))
    }

    fn spawn<W, G, F>(&mut self, g: G)
    where
        Self: Sized,
        W: Worker<Self>,
        G: FnOnce(oneshot::Receiver<()>) -> F,
        F: Future<Output = ()> + Send + 'static,
    {
        let (tx, rx) = oneshot::channel();

        self.tasks
            .entry(TypeId::of::<W>())
            .or_default()
            .push((tx, Box::new(spawn(g(rx)))));
    }

    fn worker<W>(&self) -> Option<&W>
    where
        Self: Sized,
        W: Worker<Self> + Send + Sync,
    {
        self.workers.get::<W>()
    }
}

pub struct BeeNodeBuilder<B: Backend> {
    deps: HashMap<TypeId, &'static [TypeId]>,
    worker_starts: HashMap<TypeId, Box<WorkerStart<BeeNode<B>>>>,
    worker_stops: HashMap<TypeId, Box<WorkerStop<BeeNode<B>>>>,
}

impl<B: Backend> Default for BeeNodeBuilder<B> {
    fn default() -> Self {
        Self {
            deps: HashMap::default(),
            worker_starts: HashMap::default(),
            worker_stops: HashMap::default(),
        }
    }
}

#[async_trait(?Send)]
impl<B: Backend> NodeBuilder<BeeNode<B>> for BeeNodeBuilder<B> {
    fn with_worker<W: Worker<BeeNode<B>> + 'static>(self) -> Self
    where
        W::Config: Default,
    {
        self.with_worker_cfg::<W>(W::Config::default())
    }

    fn with_worker_cfg<W: Worker<BeeNode<B>> + 'static>(mut self, config: W::Config) -> Self {
        self.deps.insert(TypeId::of::<W>(), W::dependencies());
        self.worker_starts.insert(
            TypeId::of::<W>(),
            Box::new(|node| {
                Box::pin(async move {
                    info!("Starting worker `{}`...", type_name::<W>());
                    match W::start(node, config).await {
                        Ok(w) => node.add_worker(w),
                        Err(e) => panic!("Worker `{}` failed to start: {:?}.", type_name::<W>(), e),
                    }
                })
            }),
        );
        self.worker_stops.insert(
            TypeId::of::<W>(),
            Box::new(|node| {
                Box::pin(async move {
                    info!("Stopping worker `{}`...", type_name::<W>());
                    match node.remove_worker::<W>().stop(node).await {
                        Ok(()) => {}
                        Err(e) => panic!("Worker `{}` failed to stop: {:?}.", type_name::<W>(), e),
                    }
                })
            }),
        );
        self
    }

    async fn finish(mut self) -> BeeNode<B> {
        let mut node = BeeNode {
            workers: Map::new(),
            tasks: HashMap::new(),
            resources: Map::new(),
            phantom: PhantomData,
            worker_stops: self.worker_stops,
            worker_order: TopologicalOrder::sort(self.deps),
        };

        for id in node.worker_order.clone() {
            self.worker_starts.remove(&id).unwrap()(&mut node).await;
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

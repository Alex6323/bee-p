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

use anymap::AnyMap;
use futures::{channel::oneshot, future::Future};

use std::{
    any::TypeId,
    collections::{HashMap, HashSet},
    sync::Arc,
};

pub trait Node: Send + Sync + 'static {
    fn new() -> Self;
    fn spawn<W, G, F>(&self, g: G)
    where
        Self: Sized,
        W: Worker<Self>,
        G: FnOnce(oneshot::Receiver<()>) -> F,
        F: Future<Output = ()> + Send + Sync + 'static;
}

struct NodeBuilder<N: Node> {
    deps: HashMap<TypeId, &'static [TypeId]>,
    makers: HashMap<TypeId, Box<dyn FnOnce(&N, &mut AnyMap)>>,
    anymap: AnyMap,
}

impl<N: Node + 'static> NodeBuilder<N> {
    fn with_worker<W: Worker<N> + 'static>(self) -> Self
    where
        W::Config: Default,
    {
        self.with_worker_cfg::<W>(W::Config::default())
    }

    fn with_worker_cfg<W: Worker<N> + 'static>(mut self, _config: W::Config) -> Self {
        self.deps.insert(TypeId::of::<W>(), W::DEPS);
        self.makers.insert(TypeId::of::<W>(), Box::new(|_node, _anymap| {}));
        self
    }

    fn finish(mut self) -> Arc<N> {
        let node = Arc::new(N::new());

        for id in TopologicalOrder::sort(self.deps) {
            self.makers.remove(&id).unwrap()(&node, &mut self.anymap);
        }

        node
    }
}

struct TopologicalOrder {
    graph: HashMap<TypeId, &'static [TypeId]>,
    non_visited: Vec<TypeId>,
    being_visited: HashSet<TypeId>,
    order: Vec<TypeId>,
}

impl TopologicalOrder {
    fn visit(&mut self, id: TypeId) {
        if let Some(index) = self
            .non_visited
            .iter()
            .enumerate()
            .find_map(|(index, id2)| if id == *id2 { Some(index) } else { None })
        {
            if self.being_visited.insert(id) {
                panic!("Cyclic dependency detected.");
            }

            for &id in self.graph[&id] {
                self.visit(id);
            }

            self.being_visited.remove(&id);
            self.non_visited.remove(index);
            self.order.insert(0, id);
        }
    }

    fn sort(graph: HashMap<TypeId, &'static [TypeId]>) -> Vec<TypeId> {
        let non_visited = graph.keys().copied().collect();

        let mut this = Self {
            graph,
            non_visited,
            being_visited: HashSet::new(),
            order: vec![],
        };

        while let Some(id) = this.non_visited.pop() {
            this.visit(id);
        }

        this.order
    }
}

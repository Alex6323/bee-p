use crate::worker::{Worker, WorkerId};

use std::collections::{HashMap, HashSet};

trait Node {
    fn new() -> Self;
}

struct NodeBuilder<N: Node> {
    deps: HashMap<WorkerId, &'static [WorkerId]>,
    closures: HashMap<WorkerId, Box<dyn FnOnce(&N)>>,
}

impl<N: Node> NodeBuilder<N> {
    fn with_worker<W: Worker>(mut self) -> Self {
        self.closures.insert(W::ID, Box::new(|node| {}));

        self.deps.insert(W::ID, W::DEPS);
        self
    }

    fn finish(mut self) {
        let order = TopologicalOrder::sort(self.deps);
        let node = N::new();
        for id in order {
            self.closures.remove(&id).unwrap()(&node);
        }
    }
}

struct TopologicalOrder {
    graph: HashMap<WorkerId, &'static [WorkerId]>,
    non_visited: Vec<WorkerId>,
    being_visited: HashSet<WorkerId>,
    order: Vec<WorkerId>,
}

impl TopologicalOrder {
    fn visit(&mut self, id: WorkerId) {
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

    fn sort(graph: HashMap<WorkerId, &'static [WorkerId]>) -> Vec<WorkerId> {
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

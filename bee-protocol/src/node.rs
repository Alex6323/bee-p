use crate::worker::Worker;

use std::collections::HashMap;

trait Node {}

struct BeeNode {
}

struct WorkerToAdd<N: Node> {
    deps: &'static [&'static str],
    closure: Box<dyn FnMut(&N)>,
}

struct NodeBuilder<N: Node> {
    workers: HashMap<&'static str, WorkerToAdd<N>>,
}

impl<N: Node> NodeBuilder<N> {
    fn with_worker<W: Worker>(mut self) -> Self {
        self.workers.insert(
            W::name(),
            WorkerToAdd {
                deps: W::dependencies(),
                closure: Box::new(|node| {}),
            },
        );
        self
    }

    // fn finish(mut self) -> Node {
    //     // DFS
    //     Node {
    //
    //     }
    // }
}

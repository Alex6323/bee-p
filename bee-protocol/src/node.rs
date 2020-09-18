use crate::worker::{Worker, WorkerId};

use std::collections::HashMap;

trait Node {}

struct BeeNode {}

struct WorkerToAdd<N: Node> {
    deps: &'static [WorkerId],
    closure: Box<dyn FnMut(&N)>,
}

struct NodeBuilder<N: Node> {
    workers: HashMap<WorkerId, WorkerToAdd<N>>,
}

impl<N: Node> NodeBuilder<N> {
    fn with_worker<W: Worker>(mut self) -> Self {
        self.workers.insert(
            W::ID,
            WorkerToAdd {
                deps: W::DEPS,
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

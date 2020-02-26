use crate::node::NodeMetrics;

use async_std::task::spawn;

struct Node {
    metrics: NodeMetrics,
}

impl Node {
    async fn actor() {}

    fn start() {
        spawn(Self::actor());
    }
}

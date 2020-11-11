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

use crate::{
    banner::print_banner_and_version, config::NodeConfig, plugin, storage::Backend,
    version_checker::VersionCheckerWorker,
};

use bee_common::{
    shutdown,
    shutdown_stream::ShutdownStream,
};
use bee_common_ext::{
    event::Bus,
    node::{Node, NodeBuilder, ResHandle},
    worker::Worker,
    shutdown_tokio::Shutdown,
};
use bee_network::{self, Command::ConnectEndpoint, EndpointId, Event, Network, Origin};
use bee_peering::{ManualPeerManager, PeerManager};
use bee_protocol::Protocol;

use anymap::{any::Any as AnyMapAny, Map};
use async_trait::async_trait;
use futures::{
    channel::oneshot,
    future::Future,
    stream::{Fuse, StreamExt},
};
use log::{error, info, trace, warn};
use thiserror::Error;
use tokio::spawn;

use std::{
    any::{type_name, Any, TypeId},
    collections::{HashMap, HashSet},
    marker::PhantomData,
    net::SocketAddr,
    pin::Pin,
    ops::Deref,
};

type NetworkEventStream = ShutdownStream<Fuse<flume::r#async::RecvStream<'static, Event>>>;

// TODO design proper type `PeerList`
type PeerList = HashMap<EndpointId, (flume::Sender<Vec<u8>>, oneshot::Sender<()>)>;

type WorkerStart<N> = dyn for<'a> FnOnce(&'a mut N) -> Pin<Box<dyn Future<Output = ()> + 'a>>;
type WorkerStop<N> = dyn for<'a> FnOnce(&'a mut N) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> + Send;
type ResourceRegister<N> = dyn for<'a> FnOnce(&'a mut N);

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
    shutdown: Shutdown,
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

    pub fn config(&self) -> impl Deref<Target = NodeConfig<B>> + Clone {
        self.resource()
    }

    #[allow(missing_docs)]
    pub async fn run(mut self) -> Result<(), Error> {
        info!("Running.");

        let mut network_events_stream = self.remove_resource::<NetworkEventStream>().unwrap();

        let config = self.config();
        let network = self.resource::<Network>();
        let mut runtime = NodeRuntime {
            peers: PeerList::default(),
            config: &config,
            network: &network,
            node: &self,
        };

        while let Some(event) = network_events_stream.next().await {
            trace!("Received event {}.", event);

            runtime.process_event(event);
        }

        info!("Stopping...");

        for (_, (_, shutdown)) in runtime.peers.into_iter() {
            // TODO: Should we handle this error?
            let _ = shutdown.send(());
        }

        self.stop().await.expect("Failed to properly stop node");

        info!("Stopped.");

        Ok(())
    }
}

struct NodeRuntime<'a, B: Backend> {
    peers: PeerList,
    config: &'a NodeConfig<B>,
    network: &'a Network,
    node: &'a BeeNode<B>,
}

impl<'a, B: Backend> NodeRuntime<'a, B> {
    #[inline]
    fn process_event(&mut self, event: Event) {
        match event {
            Event::EndpointAdded { epid, .. } => self.endpoint_added_handler(epid),

            Event::EndpointRemoved { epid, .. } => self.endpoint_removed_handler(epid),

            Event::EndpointConnected {
                epid,
                peer_address,
                origin,
            } => self.endpoint_connected_handler(epid, peer_address, origin),

            Event::EndpointDisconnected { epid, .. } => self.endpoint_disconnected_handler(epid),

            Event::MessageReceived { epid, message, .. } => self.endpoint_bytes_received_handler(epid, message),
            _ => warn!("Unsupported event {}.", event),
        }
    }

    #[inline]
    fn endpoint_added_handler(&self, epid: EndpointId) {
        info!("Endpoint {} has been added.", epid);

        if let Err(e) = self.network.unbounded_send(ConnectEndpoint { epid }) {
            warn!("Sending Command::Connect for {} failed: {}.", epid, e);
        }
    }

    #[inline]
    fn endpoint_removed_handler(&self, epid: EndpointId) {
        info!("Endpoint {} has been removed.", epid);
    }

    #[inline]
    fn endpoint_connected_handler(&mut self, epid: EndpointId, peer_address: SocketAddr, origin: Origin) {
        let (receiver_tx, receiver_shutdown_tx) =
            Protocol::register(self.node, &self.config.protocol, epid, peer_address, origin);

        self.peers.insert(epid, (receiver_tx, receiver_shutdown_tx));
    }

    #[inline]
    fn endpoint_disconnected_handler(&mut self, epid: EndpointId) {
        // TODO unregister ?
        if let Some((_, shutdown)) = self.peers.remove(&epid) {
            if let Err(e) = shutdown.send(()) {
                warn!("Sending shutdown to {} failed: {:?}.", epid, e);
            }
        }
    }

    #[inline]
    fn endpoint_bytes_received_handler(&mut self, epid: EndpointId, bytes: Vec<u8>) {
        if let Some(peer) = self.peers.get_mut(&epid) {
            if let Err(e) = peer.0.send(bytes) {
                warn!("Sending PeerWorkerEvent::Message to {} failed: {}.", epid, e);
            }
        }
    }
}

fn ctrl_c_listener() -> oneshot::Receiver<()> {
    let (sender, receiver) = oneshot::channel();

    tokio::spawn(async move {
        if let Err(e) = tokio::signal::ctrl_c().await {
            panic!("Failed to intercept CTRL-C: {:?}.", e);
        }

        if let Err(e) = sender.send(()) {
            panic!("Failed to send the shutdown signal: {:?}.", e);
        }
    });

    receiver
}

#[async_trait]
impl<B: Backend> Node for BeeNode<B> {
    type Builder = BeeNodeBuilder<B>;
    type Backend = B;

    async fn stop(mut self) -> Result<(), shutdown::Error>
    where
        Self: Sized,
    {
        for worker_id in self.worker_order.clone().into_iter().rev() {
            for (shutdown, task_fut) in self.tasks.remove(&worker_id).unwrap_or_default() {
                let _ = shutdown.send(());
                // TODO: Should we handle this error?
                let _ = task_fut.await; //.map_err(|e| shutdown::Error::from(worker::Error(Box::new(e))))?;
            }
            self.worker_stops.remove(&worker_id).unwrap()(&mut self).await;
            self.resource::<Bus>().purge_worker_listeners(worker_id);
        }

        Ok(())
    }

    fn register_resource<R: Any + Send + Sync>(&mut self, res: R) {
        self.resources.insert(ResHandle::new(res));
    }

    fn remove_resource<R: Any + Send + Sync>(&mut self) -> Option<R> {
        self.resources.remove::<ResHandle<R>>()?.try_unwrap()
    }

    #[track_caller]
    fn resource<R: Any + Send + Sync>(&self) -> ResHandle<R> {
        self.resources
            .get::<ResHandle<R>>()
            .unwrap_or_else(|| panic!("Unable to fetch node resource {}", type_name::<R>()))
            .clone()
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

#[derive(Error, Debug)]
pub enum Error {
    /// Occurs, when there is an error while reading the snapshot file.
    #[error("Reading snapshot file failed.")]
    SnapshotError(bee_snapshot::Error),

    /// Occurs, when there is an error while shutting down the node.
    #[error("Shutting down failed.")]
    ShutdownError(#[from] bee_common::shutdown::Error),
}

pub struct BeeNodeBuilder<B: Backend> {
    deps: HashMap<TypeId, &'static [TypeId]>,
    worker_starts: HashMap<TypeId, Box<WorkerStart<BeeNode<B>>>>,
    worker_stops: HashMap<TypeId, Box<WorkerStop<BeeNode<B>>>>,
    resource_registers: Vec<Box<ResourceRegister<BeeNode<B>>>>,
    config: NodeConfig<B>,
}

#[async_trait(?Send)]
impl<B: Backend> NodeBuilder<BeeNode<B>> for BeeNodeBuilder<B> {
    type Error = Error;
    type Config = NodeConfig<B>;

    fn new(config: Self::Config) -> Self {
        Self {
            deps: HashMap::default(),
            worker_starts: HashMap::default(),
            worker_stops: HashMap::default(),
            resource_registers: Vec::default(),
            config,
        }
        .with_resource(Bus::default())
    }

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

    fn with_resource<R: Any + Send + Sync>(mut self, res: R) -> Self {
        self.resource_registers.push(Box::new(move |node| {
            node.register_resource(res);
        }));
        self
    }

    async fn finish(mut self) -> Result<BeeNode<B>, Error> {
        print_banner_and_version();

        let mut shutdown = Shutdown::new();

        info!("Initializing network...");
        let (network, events) = bee_network::init(self.config.network.clone(), &mut shutdown).await;

        let config = self.config.clone();
        let builder = self
            .with_resource(network)
            .with_resource(ShutdownStream::new(ctrl_c_listener(), events.into_stream()))
            .with_resource(config.clone()) // TODO: Remove clone
            .with_resource(PeerList::default());

        let (builder, snapshot) = bee_snapshot::init::<BeeNode<B>>(&config.snapshot, builder)
            .await
            .map_err(Error::SnapshotError)?;

        // info!("Initializing ledger...");
        // node_builder = bee_ledger::whiteflag::init::<BeeNode<B>>(
        //     snapshot_metadata.index(),
        //     snapshot_state.into(),
        //     self.config.protocol.coordinator().clone(),
        //     node_builder,
        //     bus.clone(),
        // );

        info!("Initializing protocol...");
        let builder = Protocol::init::<BeeNode<B>>(
            config.protocol.clone(),
            config.database.clone(),
            &snapshot,
            builder,
        );

        info!("Initializing plugins...");
        // plugin::init(bus.clone());

        let mut builder = builder.with_worker::<VersionCheckerWorker>();

        let mut node = BeeNode {
            workers: Map::new(),
            tasks: HashMap::new(),
            resources: Map::new(),
            worker_stops: builder.worker_stops,
            worker_order: TopologicalOrder::sort(builder.deps),
            shutdown,
            phantom: PhantomData,
        };

        for f in builder.resource_registers {
            f(&mut node);
        }

        for id in node.worker_order.clone() {
            builder.worker_starts.remove(&id).unwrap()(&mut node).await;
        }

        // TODO: turn into worker
        info!("Starting manual peer manager...");
        spawn({
            let network = node.resource::<Network>();
            let peering_manual = config.peering.manual.clone();
            async move { ManualPeerManager::new(peering_manual).run(&network).await; }
        });

        info!("Registering events...");
        bee_snapshot::events(&node);
        // bee_ledger::whiteflag::events(&bee_node, bus.clone());
        Protocol::events(&node, config.protocol.clone());

        info!("Initialized.");

        Ok(node)
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

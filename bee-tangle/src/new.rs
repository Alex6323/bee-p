// Defined externally
type Hash = u64;
type Transaction = ();

pub struct Vertex {
    trunk: Hash,
    branch: Hash,
    solid: bool,
}

pub struct Tangle {
    vertices: DashMap<Hash, Vertex>,
    unsolid_new: mpsc::Sender<Hash>,
}

impl Tangle {
    pub async fn insert(&self, hash: Hash, v: Vertex) {
        self.vertices.write_async(hash, v).await;
        self.unsolid_new.send(hash).await;
    }

    ...
}

static mut TANGLE: AtomicPtr<Tangle> = AtomicPtr::new(ptr::null_mut());

pub fn tangle() -> &'static Tangle {
    let tangle = TANGLE.load(Ordering::Relaxed);
    if tangle.is_null() {
        panic!("Tangle cannot be null");
    } else {
        unsafe { &*tangle }
    }
}

// Solidifier

pub struct SoldifierState {
    vert_to_approvers: HashMap<Hash, Vec<Hash>>,
    missing_to_approvers: HashMap<Hash, Vec<Arc<TxHash>>>,
    unsolid_new: mpsc::Receiver<Hash>,
}

impl SoldifierState {
    pub async fn worker(mut state: SoldifierState) {
        while let Ok(hash) = state.next().await {
            // Solidification algorithm here, write back to TANGLE
        }
    }
}

// VertexRef API

pub struct VertexRef {
    hash: Hash,
    trunk: Hash,
    branch: Hash,
    tangle: &'static Tangle,
}

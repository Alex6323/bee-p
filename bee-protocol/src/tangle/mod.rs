use crate::milestone::MilestoneIndex as MsIndex;

use bee_tangle::{Tangle, TransactionRef};
use bee_transaction::{BundledTransaction as Transaction, Hash as THash};

use bitflags::bitflags;
use dashmap::{DashMap, DashSet};

use std::{
    ptr,
    sync::atomic::{AtomicBool, AtomicPtr, AtomicU32, Ordering},
};

bitflags! {
    pub struct Flags: u8 {
        const SOLID = 0b0000_0001;
        const TAIL = 0b0000_0010;
        const REQUESTED = 0b0000_0100;
        const MILESTONE = 0b0000_1000;
    }
}

impl Flags {
    pub(crate) fn is_solid(&self) -> bool {
        self.contains(Flags::SOLID)
    }

    pub(crate) fn set_solid(&mut self) {
        self.insert(Flags::SOLID);
    }

    pub(crate) fn is_tail(&self) -> bool {
        self.contains(Flags::TAIL)
    }

    pub(crate) fn set_tail(&mut self) {
        self.insert(Flags::TAIL);
    }

    pub(crate) fn is_requested(&self) -> bool {
        self.contains(Flags::REQUESTED)
    }

    pub(crate) fn set_requested(&mut self) {
        self.insert(Flags::REQUESTED);
    }

    pub(crate) fn is_milestone(&self) -> bool {
        self.contains(Flags::MILESTONE)
    }

    pub(crate) fn set_milestone(&mut self) {
        self.insert(Flags::MILESTONE);
    }
}

/// Milestone-based Tangle.
pub struct MsTangle {
    pub(crate) inner: Tangle<Flags>,
    pub(crate) milestones: DashMap<MsIndex, THash>,
    pub(crate) solid_entry_points: DashSet<THash>,
    solid_milestone_index: AtomicU32,
    last_milestone_index: AtomicU32,
    snapshot_milestone_index: AtomicU32,
}

impl MsTangle {
    pub fn new() -> Self {
        Self {
            inner: Tangle::new(),
            milestones: DashMap::new(),
            solid_entry_points: DashSet::new(),
            solid_milestone_index: AtomicU32::new(0),
            last_milestone_index: AtomicU32::new(0),
            snapshot_milestone_index: AtomicU32::new(0),
        }
    }

    pub fn insert(&'static self, transaction: Transaction, hash: THash, flags: Flags) -> (TransactionRef, bool) {
        self.inner.insert(transaction, hash, flags)
    }

    pub fn get(&'static self, hash: &THash) -> Option<TransactionRef> {
        self.inner.get_transaction(hash)
    }

    pub fn contains(&'static self, hash: &THash) -> bool {
        self.inner.vertices.contains_key(hash)
    }

    pub fn add_milestone(&'static self, index: MsIndex, hash: THash) {
        self.milestones.insert(index, hash);

        self.inner.vertices.get_mut(&hash).map(|mut vtx| {
            (*vtx.get_metadata_mut()).set_milestone();
        });
    }

    pub fn remove_milestone(&'static self, index: MsIndex) {
        self.milestones.remove(&index);
    }

    pub fn get_milestone(&'static self, index: MsIndex) -> Option<TransactionRef> {
        match self.get_milestone_hash(index) {
            None => None,
            Some(hash) => self.get(&hash),
        }
    }

    pub fn get_milestone_hash(&'static self, index: MsIndex) -> Option<THash> {
        match self.milestones.get(&index) {
            None => None,
            Some(v) => Some(*v),
        }
    }

    pub fn contains_milestone(&'static self, index: MsIndex) -> bool {
        self.milestones.contains_key(&index)
    }

    pub fn get_solid_milestone_index(&'static self) -> MsIndex {
        self.solid_milestone_index.load(Ordering::Relaxed).into()
    }

    pub fn get_last_milestone_index(&'static self) -> MsIndex {
        self.last_milestone_index.load(Ordering::Relaxed).into()
    }

    pub fn get_snapshot_milestone_index(&'static self) -> MsIndex {
        self.snapshot_milestone_index.load(Ordering::Relaxed).into()
    }

    pub fn update_solid_milestone_index(&'static self, new_index: MsIndex) {
        self.solid_milestone_index.store(*new_index, Ordering::Relaxed);
    }

    pub fn update_last_milestone_index(&'static self, new_index: MsIndex) {
        self.last_milestone_index.store(*new_index, Ordering::Relaxed);
    }

    pub fn update_snapshot_milestone_index(&'static self, new_index: MsIndex) {
        self.snapshot_milestone_index.store(*new_index, Ordering::Relaxed);
    }

    pub fn is_synced(&'static self) -> bool {
        self.get_solid_milestone_index() == self.get_last_milestone_index()
    }

    pub fn add_solid_entry_point(&'static self, hash: THash) {
        self.solid_entry_points.insert(hash);
    }

    /// Removes `hash` from the set of solid entry points.
    pub fn remove_solid_entry_point(&'static self, hash: &THash) {
        self.solid_entry_points.remove(hash);
    }

    /// Returns whether the transaction associated `hash` is a solid entry point.
    pub fn is_solid_entry_point(&'static self, hash: &THash) -> bool {
        self.solid_entry_points.contains(hash)
    }

    pub fn is_solid_transaction(&'static self, hash: &THash) -> bool {
        if self.is_solid_entry_point(hash) {
            true
        } else {
            self.inner
                .vertices
                .get(hash)
                .map(|vtx| vtx.value().get_metadata().is_solid())
                .unwrap_or(false)
        }
    }
}

static TANGLE: AtomicPtr<MsTangle> = AtomicPtr::new(ptr::null_mut());
static INITIALIZED: AtomicBool = AtomicBool::new(false);

pub fn init() {
    if !INITIALIZED.compare_and_swap(false, true, Ordering::Relaxed) {
        TANGLE.store(Box::into_raw(MsTangle::new().into()), Ordering::Relaxed);
    } else {
        panic!("Tangle already initialized");
    }
}

pub fn tangle() -> &'static MsTangle {
    let tangle = TANGLE.load(Ordering::Relaxed);
    if tangle.is_null() {
        panic!("Tangle cannot be null");
    } else {
        unsafe { &*tangle }
    }
}

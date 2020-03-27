use super::EndpointId as EpId;

use async_std::net::IpAddr;
use dashmap::DashMap;

use std::collections::HashSet;
use std::sync::atomic::{AtomicPtr, Ordering};
use std::ptr;

use log::*;


//type DashSet<T> = DashMap<T, ()>;

static WHITELIST: AtomicPtr<WhiteList> = AtomicPtr::new(ptr::null_mut());

// TODO: make it thread-safe and globally shared

pub fn init() {
    WHITELIST.store(Box::into_raw(WhiteList::new().into()), Ordering::SeqCst);
}

pub fn free() {
    let whitelist = WHITELIST.load(Ordering::SeqCst);
    if whitelist.is_null() {
        panic!("whitelist can't be null");
    } else {
        // just turn it into a box again, and let Rust do the cleaning up
        debug!("[Endp ] Deallocating whitelist");
        let _ = unsafe { Box::from_raw(WHITELIST.load(Ordering::SeqCst)) };
    }
}

pub fn get() -> &'static WhiteList {
    let whitelist = WHITELIST.load(Ordering::SeqCst);
    if whitelist.is_null() {
        panic!("whitelist can't be null");
    } else {
        unsafe { &*whitelist }
    }
}

pub struct WhiteList {
    inner: DashMap<EpId, IpAddr>,
}

impl WhiteList {
    pub fn new() -> Self {
        Self {
            inner: DashMap::with_capacity(10),
        }
    }
    pub fn insert(&self, epid: EpId, addr: IpAddr) -> bool {
        self.inner.insert(epid, addr).is_some()
    }

    pub fn remove(&self, epid: &EpId) -> bool {
        self.inner.remove(epid).is_some()
    }

    pub async fn refresh(&self) {
        todo!("implement refresh")
    }

    pub fn contains_address(&self, addr: &IpAddr) -> bool {
        self.inner.iter().any(|r| r.value() == addr)
    }
}
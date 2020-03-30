use super::EndpointId as EpId;

use async_std::net::IpAddr;
use dashmap::DashMap;

use std::sync::atomic::{AtomicPtr, Ordering};
use std::ptr;

use log::*;

static WHITELIST: AtomicPtr<WhiteList> = AtomicPtr::new(ptr::null_mut());

pub fn init() {
    WHITELIST.store(Box::into_raw(WhiteList::new().into()), Ordering::SeqCst);
}

pub fn free() {
    let whitelist = WHITELIST.load(Ordering::SeqCst);
    if whitelist.is_null() {
        panic!("whitelist can't be null");
    } else {
        trace!("[Endp ] Deallocating whitelist");

        // NOTE: let Box own the atomic ptr. When it gets dropped it will
        // make sure that the whitelist will be deallocated
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

    // TODO: re-resolve domain names
    #[allow(dead_code)]
    pub async fn refresh(&self) {
        todo!("implement refresh")
    }

    pub fn contains_address(&self, addr: &IpAddr) -> bool {
        self.inner.iter().any(|r| r.value() == addr)
    }
}
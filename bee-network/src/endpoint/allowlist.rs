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

use super::EndpointId as EpId;

use async_std::net::IpAddr;
use dashmap::DashMap;

use std::{
    ptr,
    sync::atomic::{AtomicBool, AtomicPtr, Ordering},
};

static ALLOWLIST: AtomicPtr<Allowlist> = AtomicPtr::new(ptr::null_mut());
static INITIALIZED: AtomicBool = AtomicBool::new(false);

const INITIAL_ALLOWLIST_CAPACITY: usize = 10;

pub fn init() {
    if !INITIALIZED.compare_and_swap(false, true, Ordering::Relaxed) {
        ALLOWLIST.store(Box::into_raw(Allowlist::new().into()), Ordering::Relaxed);
    } else {
        drop();
        panic!("Allowlist already initialized!");
    }
}

pub fn get() -> &'static Allowlist {
    let wl = ALLOWLIST.load(Ordering::Relaxed);
    if wl.is_null() {
        panic!("Allowlist cannot be null!");
    } else {
        unsafe { &*wl }
    }
}

pub fn drop() {
    if INITIALIZED.compare_and_swap(true, false, Ordering::Relaxed) {
        let wl = ALLOWLIST.swap(ptr::null_mut(), Ordering::Relaxed);
        if !wl.is_null() {
            unsafe { Box::from_raw(wl) };
        }
    } else {
        panic!("Allowlist already dropped!");
    }
}

pub struct Allowlist {
    inner: DashMap<EpId, IpAddr>,
}

impl Allowlist {
    pub fn new() -> Self {
        Self {
            inner: DashMap::with_capacity(INITIAL_ALLOWLIST_CAPACITY),
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

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    #[test]
    #[serial]
    fn init_get_and_drop() {
        init();
        let _ = get();
        drop();
    }

    #[test]
    #[should_panic]
    #[serial]
    fn double_init_should_panic() {
        init();
        init();
    }

    #[test]
    #[should_panic]
    #[serial]
    fn double_drop_should_panic() {
        init();
        drop();
        drop();
    }

    #[test]
    #[should_panic]
    #[serial]
    fn drop_without_init_should_panic() {
        drop();
    }

    #[test]
    #[should_panic]
    #[serial]
    fn get_without_init_should_panic() {
        let _ = get();
        drop();
    }
}

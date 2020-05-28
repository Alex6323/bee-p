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

//! `bee-tangle`

#![deny(missing_docs)]
#![allow(dead_code, unused_imports, unused_variables)]

pub use milestone::MilestoneIndex;
pub use tangle::Tangle;
pub use vertex::TransactionRef;

//mod milestone;
//mod solidifier;
mod tangle;
mod vertex;

use solidifier::SolidifierState;

use async_std::{
    sync::{channel, Arc, Barrier},
    task::spawn,
};

use bee_bundle::Hash;

use std::{
    ptr,
    sync::atomic::{AtomicBool, AtomicPtr, Ordering},
};

static TANGLE: AtomicPtr<Tangle<u8>> = AtomicPtr::new(ptr::null_mut());
static INITIALIZED: AtomicBool = AtomicBool::new(false);

const SOLIDIFIER_CHAN_CAPACITY: usize = 1000;

/// Initializes the Tangle singleton.
pub fn init() {
    if !INITIALIZED.compare_and_swap(false, true, Ordering::Relaxed) {
        let (sender, receiver) = flume::bounded::<Option<Hash>>(SOLIDIFIER_CHAN_CAPACITY);

        let drop_barrier = async_std::sync::Arc::new(Barrier::new(2));

        TANGLE.store(
            Box::into_raw(Tangle::new(sender, drop_barrier.clone()).into()),
            Ordering::Relaxed,
        );

        spawn(SolidifierState::new(receiver, drop_barrier).run());
    } else {
        drop();
        panic!("Already initialized");
    }
}

/// Returns the singleton instance of the Tangle.
pub fn tangle() -> &'static Tangle<u8> {
    let tangle = TANGLE.load(Ordering::Relaxed);
    if tangle.is_null() {
        panic!("Tangle cannot be null");
    } else {
        unsafe { &*tangle }
    }
}

/// Drops the Tangle singleton.
pub fn drop() {
    if INITIALIZED.compare_and_swap(true, false, Ordering::Relaxed) {
        tangle().shutdown();

        let tangle = TANGLE.swap(ptr::null_mut(), Ordering::Relaxed);
        if !tangle.is_null() {
            let _ = unsafe { Box::from_raw(tangle) };
        }
    } else {
        panic!("Already dropped");
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
        let _ = tangle();
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
        let _ = tangle();
        drop();
    }
}

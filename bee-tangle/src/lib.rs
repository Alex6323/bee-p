//! `bee-tangle`

#![deny(missing_docs)]
#![allow(dead_code, unused_imports, unused_variables)]

pub use milestone::MilestoneIndex;
pub use tangle::Tangle;
pub use vertex::TransactionRef;

mod milestone;
mod solidifier;
mod tangle;
mod vertex;

use solidifier::SolidifierState;

use async_std::{
    sync::channel,
    task::spawn,
};

use bee_bundle::Hash;

use std::{
    ptr,
    sync::atomic::{
        AtomicBool,
        AtomicPtr,
        Ordering,
    },
};

static TANGLE: AtomicPtr<Tangle> = AtomicPtr::new(ptr::null_mut());
static INITIALIZED: AtomicBool = AtomicBool::new(false);

const SOLIDIFIER_CHAN_CAPACITY: usize = 1000;

/// Initializes the Tangle singleton.
pub fn init() {
    if !INITIALIZED.compare_and_swap(false, true, Ordering::Relaxed) {
        let (sender, receiver) = flume::bounded::<Hash>(SOLIDIFIER_CHAN_CAPACITY);

        let solidifier_state = SolidifierState::new(receiver);

        TANGLE.store(Box::into_raw(Tangle::new(sender).into()), Ordering::Relaxed);

        spawn(solidifier::run(solidifier_state));
    } else {
        drop();
        panic!("Already initialized");
    }
}

/// Returns the singleton instance of the Tangle.
pub fn tangle() -> &'static Tangle {
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
        let tangle = TANGLE.swap(ptr::null_mut(), Ordering::Relaxed);
        if tangle.is_null() {
            return;
        } else {
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

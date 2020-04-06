//! `bee-tangle`

#![deny(missing_docs)]
#![allow(dead_code, unused_imports, unused_variables)]

pub use milestone::MilestoneIndex;
pub use tangle::Tangle;

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

/// A temporary redefinition of `bee-bundle::Hash`.
pub type TransactionId = Hash;

static TANGLE: AtomicPtr<Tangle> = AtomicPtr::new(ptr::null_mut());
static INITIALIZED: AtomicBool = AtomicBool::new(false);

const SOLIDIFIER_CHAN_CAPACITY: usize = 1000;

/// Initializes the Tangle singleton.
pub fn init() {
    if !INITIALIZED.compare_and_swap(false, true, Ordering::Relaxed) {
        let (sender, receiver) = channel::<Hash>(SOLIDIFIER_CHAN_CAPACITY);

        let solidifier_state = SolidifierState::new(receiver);

        let tangle = TANGLE.load(Ordering::Relaxed);
        if !tangle.is_null() {
            panic!("Already initialized");
        } else {
            TANGLE.store(Box::into_raw(Tangle::new(sender).into()), Ordering::Relaxed);
        }

        spawn(solidifier::run(solidifier_state));
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

/// Deallocates the Tangle singleton.
pub fn exit() {
    if INITIALIZED.compare_and_swap(true, false, Ordering::Relaxed) {
        let tangle = TANGLE.swap(ptr::null_mut(), Ordering::Relaxed);
        if tangle.is_null() {
            return;
        } else {
            let _ = unsafe { Box::from_raw(tangle) };
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    #[test]
    #[serial]
    fn init_and_exit() {
        init();
        let _ = tangle();
        exit();
    }

    #[test]
    #[serial]
    fn double_init_double_exit() {
        init();
        init();
        let _ = tangle();
        exit();
        exit();
    }
}

mod solidifier;
mod tangle;
mod vertex;

use solidifier::{
    worker,
    SoldifierState,
};
use tangle::Tangle;

use async_std::sync::{
    channel,
    Receiver,
    Sender,
};

use bee_bundle::{
    Hash,
    Transaction,
};

use std::{
    ptr,
    sync::atomic::{
        AtomicPtr,
        Ordering,
    },
};

pub type TransactionId = Hash;
pub type MilestoneIndex = usize;

static TANGLE: AtomicPtr<Tangle> = AtomicPtr::new(ptr::null_mut());

const SOLIDIFIER_CHAN_CAPACITY: usize = 1000;

pub fn tangle() -> &'static Tangle {
    let tangle = TANGLE.load(Ordering::Relaxed);
    if tangle.is_null() {
        panic!("Tangle cannot be null");
    } else {
        unsafe { &*tangle }
    }
}

pub fn init() {
    let (sender, receiver) = channel::<Hash>(SOLIDIFIER_CHAN_CAPACITY);

    TANGLE.store(Box::into_raw(Tangle::new(sender).into()), Ordering::Acquire);
}

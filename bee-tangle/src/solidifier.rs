use std::collections::HashMap;

use async_std::{
    prelude::*,
    sync::{
        Arc,
        Receiver,
    },
};

use bee_bundle::Hash;

pub struct SoldifierState {
    vert_to_approvers: HashMap<Hash, Vec<Hash>>,
    missing_to_approvers: HashMap<Hash, Vec<Arc<Hash>>>,
    unsolid_new: Receiver<Hash>,
}

pub async fn worker(mut state: SoldifierState) {
    while let Some(hash) = state.unsolid_new.next().await {
        // Solidification algorithm here, write back to TANGLE
    }
}

use crate::cores::Cores;
use crate::difficulty::Difficulty;
use crate::nonce::NonceTrits;

use std::sync::{Arc, RwLock};

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum PearlDiverState {
    Created,
    Searching,
    Cancelled,
    Completed(Option<NonceTrits>),
}

#[derive(Clone)]
pub struct PearlDiver {
    cores: Cores,
    difficulty: Difficulty,
    state: Arc<RwLock<PearlDiverState>>,
}

impl PearlDiver {
    pub fn new(cores: Cores, difficulty: Difficulty) -> Self {
        Self {
            cores,
            difficulty,
            ..Self::default()
        }
    }
}

impl Default for PearlDiver {
    fn default() -> Self {
        Self {
            cores: Cores::default(),
            difficulty: Difficulty::default(),
            state: Arc::new(RwLock::new(PearlDiverState::Created)),
        }
    }
}

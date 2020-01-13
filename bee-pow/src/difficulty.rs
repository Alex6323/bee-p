#[derive(Clone)]
pub struct Difficulty(pub(self) usize);

use common::constants::{HASH_TRIT_LEN, NETWORK_DIFFICULTY};

impl Default for Difficulty {
    fn default() -> Self {
        Self(NETWORK_DIFFICULTY)
    }
}

impl From<usize> for Difficulty {
    fn from(difficulty: usize) -> Self {
        let max_difficulty = HASH_TRIT_LEN;
        if difficulty > max_difficulty {
            Self(max_difficulty)
        } else {
            Self(difficulty)
        }
    }
}

impl std::ops::Deref for Difficulty {
    type Target = usize;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

use bee_common::constants::{HASH_TRIT_LEN, MAINNET_DIFFICULTY, DEVNET_DIFFICULTY, SPAMNET_DIFFICULTY};

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Difficulty(pub(self) usize);

impl Difficulty {
    pub fn mainnet() -> Self {
        Self(MAINNET_DIFFICULTY)
    }

    pub fn devnet() -> Self {
        Self(DEVNET_DIFFICULTY)
    }

    pub fn spamnet() -> Self {
        Self(SPAMNET_DIFFICULTY)
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

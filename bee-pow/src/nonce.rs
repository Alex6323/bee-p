use bee_common::constants::NONCE_TRIT_LEN as NONCE_LEN;
use bee_common::Trit;

#[derive(Copy)]
pub struct NonceTrits(pub(crate) [Trit; NONCE_LEN]);

impl NonceTrits {
    pub fn to_vec(&self) -> Vec<i8> {
        self.0.to_vec()
    }

    pub fn as_slice(&self) -> &[i8] {
        &self.0[..]
    }
}

impl Default for NonceTrits {
    fn default() -> Self {
        Self([0i8; NONCE_LEN])
    }
}

impl Eq for NonceTrits {}
impl PartialEq for NonceTrits {
    fn eq(&self, other: &Self) -> bool {
        self.0.iter().zip(other.0.iter()).all(|(&i, &j)| i == j)
    }
}

impl Clone for NonceTrits {
    fn clone(&self) -> Self {
        let mut nonce = [0_i8; NONCE_LEN];
        nonce.copy_from_slice(&self.0);

        Self(nonce)
    }
}

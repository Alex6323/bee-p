use crate::binary::{Ed25519Seed, Error};

use bee_ternary::{TritBuf, T5B1Buf, T1B1Buf};

pub enum Seed {
    Ed25519(Ed25519Seed),
    Wots(bee_signing::ternary::seed::Seed),
}

impl Seed {
    pub fn from_ed25519_bytes(bytes: &[u8]) -> Result<Self, Error> {
        Ok(Seed::Ed25519(
            Ed25519Seed::from_bytes(bytes)?,
        ))
    }

    pub fn from_wots_tritbuf(trits: &TritBuf<T5B1Buf>) -> Result<Self, Error> {
        if trits.as_i8_slice().len() != 49 {
            return Err(Error::InvalidLength(49));
        }
        let trits: TritBuf<T1B1Buf> = trits.encode();
        Ok(Seed::Wots(
            bee_signing::ternary::seed::Seed::from_trits(trits).map_err(|_| Error::ConvertError)?,
        ))
    }
}

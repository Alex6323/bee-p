use bee_ternary::{
    TritBuf,
    Trits,
};

pub trait Seed {
    type Error;

    fn new() -> Self;

    fn from_buf(buf: TritBuf) -> Result<Self, Self::Error>
    where
        Self: Sized;

    fn as_bytes(&self) -> &[i8];

    fn trits(&self) -> &Trits;
}

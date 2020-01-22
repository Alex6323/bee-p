use crate::{
    Trits,
    TritsMut,
    TritsBuf,
};

pub(crate) trait Sealed {}

impl<'a> Sealed for Trits<'a> {}
impl<'a> Sealed for TritsMut<'a> {}
impl Sealed for TritsBuf {}

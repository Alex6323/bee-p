use crate::bigint::common::{
    BigEndian,
    LittleEndian,
};

pub trait Sealed {}

impl Sealed for BigEndian {}
impl Sealed for LittleEndian {}

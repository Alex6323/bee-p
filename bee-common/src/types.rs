use crate::errors::PrototypeError;

pub type PrototypeResult = Result<(), PrototypeError>;

pub type Trit = i8; // {-01}
pub type Tryte = char; // {9ABC...Z}

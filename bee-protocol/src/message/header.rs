pub(crate) const HEADER_TYPE_SIZE: usize = 1;
const HEADER_LENGTH_SIZE: usize = 2;
pub(crate) const HEADER_SIZE: usize = HEADER_TYPE_SIZE + HEADER_LENGTH_SIZE;

pub(crate) type Header = [u8; HEADER_SIZE];

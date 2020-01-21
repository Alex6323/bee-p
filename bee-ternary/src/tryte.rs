pub enum Tryte {}

pub const TRYTE_ALPHABET: [char; 27] = [
    '9', 'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R',
    'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z',
];

pub trait IsTryte {
    fn is_tryte(&self) -> bool;
}

impl IsTryte for char {
    fn is_tryte(&self) -> bool {
        *self == '9' || (*self >= 'A' && *self <= 'Z')
    }
}

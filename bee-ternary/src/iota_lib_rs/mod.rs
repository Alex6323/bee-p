pub mod iota_units;
pub mod trinary;
pub mod trytes_converter;
pub mod unit_converter;

type Result<T> = ::std::result::Result<T, failure::Error>;

/// Converts a slice of trits into a numeric value
pub fn value(trits: &[i8]) -> i8 {
    trits.iter().rev().fold(0, |acc, trit| acc * 3 + *trit)
}

/// Converts a slice of trits into a numeric value in i64
pub fn long_value(trits: &[i8]) -> i64 {
    trits
        .iter()
        .rev()
        .fold(0, |acc, trit| acc * 3 + i64::from(*trit))
}

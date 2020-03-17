use crate::TritBuf;

pub fn trytes_to_trits_buf(trytes: &str) -> TritBuf {
    let mut trits_buf_internal = Vec::new();
    for tryte in trytes.trim().chars() {
        let trits = match tryte {
            '9' => &[ 0, 0, 0],
            'A' => &[ 1, 0, 0],
            'B' => &[-1, 1, 0],
            'C' => &[ 0, 1, 0],
            'D' => &[ 1, 1, 0],
            'E' => &[-1,-1, 1],
            'F' => &[ 0,-1, 1],
            'G' => &[ 1,-1, 1],
            'H' => &[-1, 0, 1],
            'I' => &[ 0, 0, 1],
            'J' => &[ 1, 0, 1],
            'K' => &[-1, 1, 1],
            'L' => &[ 0, 1, 1],
            'M' => &[ 1, 1, 1],
            'N' => &[-1,-1,-1],
            'O' => &[ 0,-1,-1],
            'P' => &[ 1,-1,-1],
            'Q' => &[-1, 0,-1],
            'R' => &[ 0, 0,-1],
            'S' => &[ 1, 0,-1],
            'T' => &[-1, 1,-1],
            'U' => &[ 0, 1,-1],
            'V' => &[ 1, 1,-1],
            'W' => &[-1,-1, 0],
            'X' => &[ 0,-1, 0],
            'Y' => &[ 1,-1, 0],
            'Z' => &[-1, 0, 0],
            x => {
                panic!("unexpected character: >>{}<<", x)
            }
        };
        trits_buf_internal.extend_from_slice(trits);
    }
    TritBuf::from_i8_unchecked(&trits_buf_internal)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::T1B1Buf;

    const TRYTES_IN: &str = "AZN9";
    const BYTES_EXPECTED: &[i8] = &[1, 0, 0, -1, 0, 0, -1, -1, -1, 0, 0, 0];

    #[test]
    fn correct_trytes_to_trits_conversion() {
        let trits_converted = trytes_to_trits_buf(TRYTES_IN);
        let trits_expected = TritBuf::<T1B1Buf>::from_i8_unchecked(&BYTES_EXPECTED).unwrap();
        assert_eq!(trits_expected, trits_converted);
    }
}

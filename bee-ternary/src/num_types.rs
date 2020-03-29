use std::convert::TryFrom;
use crate::raw::{RawEncoding, RawEncodingBuf};

pub use crate::{
    trit::{Trit, Utrit, Btrit, ShiftTernary},
    t1b1::{T1B1, T1B1Buf},
    TritBuf,
};

use std::convert::{From, Into};


const RADIX : u8 = 3;
const NUM_TRITS_FOR_NUM_TYPE : usize = 27;
const MAX_TRITS_IN_I64 : usize = (0.63 * 64 as f32) as usize + 1; // (log2/log3)*64


fn min_trits(value: u64) -> usize {
    let mut num = 1;
    let mut vp : u64 = 1;

    let mut value = value;
    let mut num = 1;
    while value > vp {
        vp = vp * RADIX as u64 + 1;
        num = num + 1;
    }

    num
}




//TODO - generalize over all encodings
//Impl copied from:
// [https://github.com/iotaledger/iota_common/blob/master/common/trinary/trit_long.c#L62]
impl From<i64> for TritBuf<T1B1Buf> {
    fn from(value: i64) -> Self {

        let negative = value < 0;
        // Edge case where value == i64::MIN. In this case,
        // "abs" cannot return a value greater than i64::MAX
        // Which it should since the range is not symmetrical
        // so we "force" the (u64) value explicitly
        let mut value_abs = match value {
            std::i64::MIN => (std::i64::MAX as u64 + 1),
            _ => value.abs() as u64

        } ;

        let size = min_trits(value_abs);
        let mut buf = Self::zeros(size);

        let mut last_pos = 0;
        for pos in 0..size {
            if value_abs == 0 {
                break;
            }
            let mut curr_trit = ((value_abs + 1) % (RADIX as u64)) as i8 - 1;
            if negative {
                curr_trit = -curr_trit;
            }

            buf.set(pos, Btrit::try_from(curr_trit).unwrap());

            value_abs = value_abs + 1;
            value_abs = value_abs / RADIX as u64;

            last_pos = pos;
        }

        buf
    }
}


//TODO - generalize over all encodings
//Impl copied from:
// [https://github.com/iotaledger/iota_common/blob/1b56a5282933fb674181001630e7b2e2c33b5eea/common/trinary/trit_long.c#L31]
impl From<TritBuf<T1B1Buf>> for i64 {
    fn from(trits: TritBuf<T1B1Buf>) -> i64 {
        if trits.len() == 0 {
            return 0
        }

        if trits.len() > MAX_TRITS_IN_I64 {
            panic!("Can not convert buffer of len: {} to an i64 type", trits.len());
        }

        let mut accum : i128 = 0;
        let mut end = trits.len();
        for index in (0..end).rev() {
            let trit_value : i128 = match unsafe {trits.get_unchecked(index)} {

                Btrit::Zero => 0,
                Btrit::NegOne => -1,
                Btrit::PlusOne => 1,
            };
            accum = (accum * RADIX as i128) as i128 + trit_value;
        }
        accum as i64
    }

}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::T1B1Buf;

    #[test]
    fn convert_1_to_trits() {
        let num = 1;
        let buff  = TritBuf::<T1B1Buf>::try_from(num);
        let converted_num = i64::try_from(buff.unwrap()).unwrap();
        assert_eq!(converted_num, num);
    }

    #[test]
    fn convert_neg_1_to_trits() {
        let num = -1;
        let buff  = TritBuf::<T1B1Buf>::try_from(num);
        let converted_num = i64::try_from(buff.unwrap()).unwrap();
        assert_eq!(converted_num, num);
    }

    #[test]
    fn convert_i64_max_to_trits() {
        let num = std::i64::MAX ;
        let buff  = TritBuf::<T1B1Buf>::try_from(num);
        let converted_num = i64::try_from(buff.unwrap()).unwrap();
        assert_eq!(converted_num, num);
    }

    #[test]
    fn convert_i64_min_to_trits() {
        let num = std::i64::MIN ;
        let buff  = TritBuf::<T1B1Buf>::try_from(num);
        let converted_num = i64::try_from(buff.unwrap()).unwrap();
        assert_eq!(converted_num, num);
    }
}

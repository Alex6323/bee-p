use std::convert::TryFrom;
use bee_ternary::{
    T1B1Buf,
    TritBuf,
    TryteBuf,
    bigint::{
        T243,
        I384,
        common::{
            BigEndian,
            U8Repr,
        },
    },
};

const INPUT_TRYTES: &'static str = "EMIDYNHBWMBCXVDEFOFWINXTERALUKYYPPHKP9JJFGJEIUY9MUDVNFZHMMWZUYUSWAIOWEVTHNWMHANBH";

const TRYTES_AS_I384_BE_U8: [u8; 48] = [
    236,  51,  87, 194, 177, 242, 107, 101,
    103, 168,   5,  66, 166,  81,  89, 243,
    253, 197, 196, 167, 255,  13,   7, 255,
     82, 193,  78, 211, 157, 243, 205, 238,
    142,  59,  98,  37,  11,   4,  89,  43,
    160, 190, 239, 144, 158,  28,  67,  19
];

#[test]
fn trytes_to_i384_be_u8() {
    let trytes = TryteBuf::try_from_str(INPUT_TRYTES);
    assert!(trytes.is_ok());
    let trytes = trytes.unwrap();
    let trit_buf: TritBuf<T1B1Buf> = trytes
        .as_trits()
        .encode();
    let t243 = T243::from_trit_buf(trit_buf);
    let t242 = t243.into_t242();

    let converted_i384 = I384::<BigEndian, U8Repr>::try_from(t242);
    assert!(converted_i384.is_ok());
    let converted_i384 = converted_i384.unwrap();

    let expected_i384 = I384::<BigEndian, U8Repr>::from_array(TRYTES_AS_I384_BE_U8);
    assert_eq!(converted_i384, expected_i384);
}

use std::convert::TryInto;

use bee_ternary::{
    Btrit,
    Utrit,
    bigint::{
        common::{
            LittleEndian,
            U32Repr,
        },
        T243,
        I384,
        U384,
    },
};


// #[test]
// fn ut243_all_ones_as_u384_to_self() {
//     println!("cloning original");
//     let original = crypto::u384::LE_U32_UT243_HALF_MAX.clone();
//     println!("constructing ternary");
//     let ternary = UT243::from(original);
//     let roundtripped = TryInto::<U384<LittleEndian, U32Repr>>::try_into(ternary);
//     assert!(roundtripped.is_ok());
//     assert_eq!(roundtripped.unwrap(), original);
// }

macro_rules! binary_to_ternary_roundtrip {
    ( ( $($binary_type:tt)* ), ( $($ternary_type:tt)* ), $testname:ident, $val_fn:ident ) => {
        #[test]
        fn $testname() {
            let original = $($binary_type)*::$val_fn();
            let ternary = $($ternary_type)*::from(original);
            let roundtripped = TryInto::<$($binary_type)*>::try_into(ternary);
            assert!(roundtripped.is_ok());
            assert_eq!(roundtripped.unwrap(), original);
        }
    };

    ( ( $( $binary_type:tt )* ),
      ( $( $ternary_type:tt )* ),
      [ $testname:ident, $val_fn:ident ]
      $(,)?
    ) => {
        binary_to_ternary_roundtrip!(( $($binary_type)* ), ( $($ternary_type)* ), $testname, $val_fn);
    };

    ( ( $( $binary_type:tt )* ),
      ( $( $ternary_type:tt )* ),
      [ $testname:ident, $val_fn:ident ],
      $( [ $tail_testname:ident, $tail_val_fn:ident ] ),+
      $(,)?
    ) => {
        binary_to_ternary_roundtrip!(
            ( $($binary_type)* ),
            ( $($ternary_type)* ),
            [ $testname, $val_fn ]
        );
        binary_to_ternary_roundtrip!(( $($binary_type)* ), ( $($ternary_type)* ), $( [$tail_testname, $tail_val_fn] ),+);
    };

    ( $modname:ident,
      ( $( $binary_type:tt )* ),
      ( $( $ternary_type:tt )* ),
      $( [ $testname:ident, $val_fn:ident ] ),+
      $(,)?
    ) => {
        mod $modname {
            use super::*;
            binary_to_ternary_roundtrip!(( $($binary_type)* ), ( $($ternary_type)* ), $( [$testname, $val_fn] ),+);
        }
    };
}

binary_to_ternary_roundtrip!(
    u384,
    ( U384::<LittleEndian, U32Repr> ),
    ( T243::<Utrit> ),
    [ zero_to_self, zero ],
    [ one_to_self, one ], 
    [ max_to_self, max ],
);

binary_to_ternary_roundtrip!(
    i384,
    ( I384::<LittleEndian, U32Repr> ),
    ( T243::<Btrit> ),
    [ zero_to_self, zero ],
    [ one_to_self, one ], 
    [ two_to_self, two ], 
    [ neg_one_to_self, neg_one],
    [ neg_two_to_self, neg_two],
    [ max_to_self, max ],
    [ min_to_self, min ],
);

// test_binary_to_ternary_roundtrip!(
    // [le_u32_ternary_0, LE_U32_TERNARY_0, LittleEndian],
    // [be_u32_0, BigEndian, zero],
    // [be_u32_1, BigEndian, one],
    // [be_u32_neg_1, BigEndian, neg_one],
    // [be_u32_max, BigEndian, max],
    // [be_u32_min, BigEndian, min],
    // [be_u32_2, BigEndian, two],
    // [be_u32_neg_2, BigEndian, neg_two],
    // [le_u32_0, LittleEndian, zero],
    // [le_u32_1, LittleEndian, one],
    // [le_u32_neg_1, LittleEndian, neg_one],
    // [le_u32_max, LittleEndian, max],
    // [le_u32_min, LittleEndian, min],
    // [le_u32_2, LittleEndian, two],
    // [le_u32_neg_2, LittleEndian, neg_two],
// );

use std::convert::TryInto;

use bee_ternary::{
    Utrit,
    Btrit,
    bigint::{
        T242,
        T243,
        I384,
        U384,
        common::{
            LittleEndian,
            U32Repr,
        },
    },
};

#[test]
fn t243_max_exceeds_u384_range() {
    let t243_max = T243::<Btrit>::max();
    let error = TryInto::<I384<LittleEndian, U32Repr>>::try_into(t243_max);
    assert!(error.is_err());
}

// #[test]
// fn t243_min_exceeds_u384_range() {
//     let t243_min = T243::min();
//     let error = TryInto::<I384<LittleEndian, U32Repr>>::try_into(t243_min);
//     assert!(error.is_err());
// }

// #[test]
// fn u384_max_in_t243_is_u384_max_in_t243() {
//     let converted = TryInto::<I384<LittleEndian, U32Repr>>::try_into(t243::I384_MAX.clone());
//     assert!(converted.is_ok());
//     let roundtripped: T243 = converted.unwrap().into();
//     assert_eq!(roundtripped, *t243::I384_MAX);
// }

// #[test]
// fn u384_min_in_t243_is_u384_min_in_t243() {
//     let converted = TryInto::<I384<LittleEndian, U32Repr>>::try_into(t243::I384_MIN.clone());
//     assert!(converted.is_ok());
//     let roundtripped: T243 = converted.unwrap().into();
//     assert_eq!(roundtripped, *t243::I384_MIN);
// }

macro_rules! ternary_roundtrip {
    ( @basecase: ($($ternary_type:tt)*), ($($binary_type:tt)*), $testname:ident, $val_fn:ident ) => {
        #[test]
        fn $testname() {
            let original = $($ternary_type)*::$val_fn();
            let converted = Into::<$($binary_type)*>::into(original.clone());
            let roundtripped = TryInto::<$($ternary_type)*>::try_into(converted);
            assert!(roundtripped.is_ok());
            assert_eq!(roundtripped.unwrap(), original);
        }
    };

    ( ( $($ternary_type:tt)* )
      <>
      ( $($binary_type:tt)* )
      =>
      [$testname:ident, $val_fn:ident] $(,)?
    ) => {
        ternary_roundtrip!(@basecase: ($($ternary_type)*), ($($binary_type)*), $testname, $val_fn);
    };

    ( ( $($ternary_type:tt)* )
      <>
      ( $($binary_type:tt)* )
      =>
      [$testname:ident, $val_fn:ident],
      $( [$rest_testname:ident, $rest_val_fn:ident] ),+ $(,)?
    ) => {
        ternary_roundtrip!( ( $($ternary_type)* ) <> ( $($binary_type)* ) => [$testname, $val_fn] );
        ternary_roundtrip!(
            ($($ternary_type)*) <> ($($binary_type)*)
            =>
            $([$rest_testname, $rest_val_fn]),+);
    };

    ( $modname:ident:
      ( $($ternary_type:tt)* )
      <>
      ( $($binary_type:tt)* )
      =>
      $( [$testname:ident, $val_fn:ident] ),+ $(,)?
    ) => {
        mod $modname {
            use super::*;

            ternary_roundtrip!(
              ( $($ternary_type)* )
              <>
              ( $($binary_type)* )
              =>
              $( [$testname, $val_fn] ),+
            );
        }
    };
}

ternary_roundtrip!(
    t242_btrit: (T242::<Btrit>) <> (I384<LittleEndian, U32Repr>)
    =>
    [zero_is_zero, zero],
    [one_is_one, one],
    [neg_one_is_neg_one, neg_one],
    [two_is_two, two],
    [neg_two_is_neg_two, neg_two],
    [max_is_max, max],
);

ternary_roundtrip!(
    t242_utrit: (T242::<Utrit>) <> (U384<LittleEndian, U32Repr>)
    =>
    [zero_is_zero, zero],
    [one_is_one, one],
    [two_is_two, two],
    [max_is_max, max],
);

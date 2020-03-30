use lazy_static::lazy_static;

use crate::{
    Btrit,
    Utrit,
    bigint::{
        T242,
        T243,
        U384,
        common::{
            LittleEndian,
            U32Repr,
        },
    },
};

lazy_static! {
    pub static ref BTRIT_ZERO: T242<Btrit> = T242::<Btrit>::zero();
    pub static ref BTRIT_ONE: T242<Btrit> = T242::<Btrit>::one();
    pub static ref BTRIT_NEG_ONE: T242<Btrit> = T242::<Btrit>::one();

    pub static ref UTRIT_ZERO: T242<Utrit> = T242::<Utrit>::zero();
    pub static ref UTRIT_ONE: T242<Utrit> = T242::<Utrit>::one();
    pub static ref UTRIT_NEG_ONE: T242<Utrit> = T242::<Utrit>::one();

    pub static ref UTRIT_U384_MAX: T243<Utrit> = {
        let u384_max = U384::<LittleEndian, U32Repr>::max();
        From::from(u384_max)
    };

    pub static ref UTRIT_U384_MAX_HALF: T243<Utrit> = {
        let mut u384_max = U384::<LittleEndian, U32Repr>::max();
        u384_max.divide_by_two();
        From::from(u384_max)
    };
}

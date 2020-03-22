pub struct Offset {
    pub start: usize,
    pub length: usize,
}

pub struct Field {
    pub trit_offset: Offset,
    pub tryte_offset: Offset,
}

impl Field {
    pub fn byte_start(&self) -> usize {
        self.trit_offset.start / 5
    }

    pub fn byte_length(&self) -> usize {
        if self.trit_offset.length % 5 == 0 {
            self.trit_offset.length / 5
        } else {
            self.trit_offset.length / 5 + 1
        }
    }
}

macro_rules! offsets_from_trits {
    ($start:expr, $length:expr) => {
        Field {
            trit_offset: Offset {
                start: $start,
                length: $length,
            },
            tryte_offset: Offset {
                start: $start / 3,
                length: $length / 3,
            },
        }
    };
}

macro_rules! offsets_from_previous_field {
    ($prev:expr, $length:expr) => {
        Field {
            trit_offset: Offset {
                start: ($prev).trit_offset.start + ($prev).trit_offset.length,
                length: $length,
            },
            tryte_offset: Offset {
                start: (($prev).trit_offset.start + ($prev).trit_offset.length) / 3,
                length: $length / 3,
            },
        }
    };
}

pub const IOTA_SUPPLY: i64 = 2779530283277761;
pub const TRYTE_ZERO: char = '9';

pub const PAYLOAD_TRIT_LEN: usize = 6561;
pub const ADDRESS_TRIT_LEN: usize = 243;
pub const VALUE_TRIT_LEN: usize = 81;
pub const TAG_TRIT_LEN: usize = 81;
pub const TIMESTAMP_TRIT_LEN: usize = 27;
pub const INDEX_TRIT_LEN: usize = 27;
pub const HASH_TRIT_LEN: usize = 243;
pub const NONCE_TRIT_LEN: usize = 81;

pub const PAYLOAD: Field = offsets_from_trits!(0, PAYLOAD_TRIT_LEN);
pub const ADDRESS: Field = offsets_from_previous_field!(PAYLOAD, ADDRESS_TRIT_LEN);
pub const VALUE: Field = offsets_from_previous_field!(ADDRESS, VALUE_TRIT_LEN);
pub const OBSOLETE_TAG: Field = offsets_from_previous_field!(VALUE, TAG_TRIT_LEN);
pub const TIMESTAMP: Field = offsets_from_previous_field!(OBSOLETE_TAG, TIMESTAMP_TRIT_LEN);
pub const INDEX: Field = offsets_from_previous_field!(TIMESTAMP, INDEX_TRIT_LEN);
pub const LAST_INDEX: Field = offsets_from_previous_field!(INDEX, INDEX_TRIT_LEN);
pub const BUNDLE: Field = offsets_from_previous_field!(LAST_INDEX, HASH_TRIT_LEN);
pub const TRUNK: Field = offsets_from_previous_field!(BUNDLE, HASH_TRIT_LEN);
pub const BRANCH: Field = offsets_from_previous_field!(TRUNK, HASH_TRIT_LEN);
pub const TAG: Field = offsets_from_previous_field!(BRANCH, TAG_TRIT_LEN);
pub const ATTACHMENT_TS: Field = offsets_from_previous_field!(TAG, TIMESTAMP_TRIT_LEN);
pub const ATTACHMENT_LBTS: Field = offsets_from_previous_field!(ATTACHMENT_TS, TIMESTAMP_TRIT_LEN);
pub const ATTACHMENT_UBTS: Field = offsets_from_previous_field!(ATTACHMENT_LBTS, TIMESTAMP_TRIT_LEN);
pub const NONCE: Field = offsets_from_previous_field!(ATTACHMENT_UBTS, NONCE_TRIT_LEN);

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn add_up_to_transaction_trit_length() {
        let total_trit_length = PAYLOAD.trit_offset.length
            + ADDRESS.trit_offset.length
            + VALUE.trit_offset.length
            + OBSOLETE_TAG.trit_offset.length
            + TIMESTAMP.trit_offset.length
            + INDEX.trit_offset.length
            + LAST_INDEX.trit_offset.length
            + BUNDLE.trit_offset.length
            + TRUNK.trit_offset.length
            + BRANCH.trit_offset.length
            + TAG.trit_offset.length
            + ATTACHMENT_TS.trit_offset.length
            + ATTACHMENT_LBTS.trit_offset.length
            + ATTACHMENT_UBTS.trit_offset.length
            + NONCE.trit_offset.length;

        assert_eq!(total_trit_length, TRANSACTION_TRIT_LEN);
    }

    #[test]
    fn add_up_to_transaction_tryte_length() {
        let total_tryte_length = PAYLOAD.tryte_offset.length
            + ADDRESS.tryte_offset.length
            + VALUE.tryte_offset.length
            + OBSOLETE_TAG.tryte_offset.length
            + TIMESTAMP.tryte_offset.length
            + INDEX.tryte_offset.length
            + LAST_INDEX.tryte_offset.length
            + BUNDLE.tryte_offset.length
            + TRUNK.tryte_offset.length
            + BRANCH.tryte_offset.length
            + TAG.tryte_offset.length
            + ATTACHMENT_TS.tryte_offset.length
            + ATTACHMENT_LBTS.tryte_offset.length
            + ATTACHMENT_UBTS.tryte_offset.length
            + NONCE.tryte_offset.length;

        assert_eq!(total_tryte_length, TRANSACTION_TRYT_LEN);
    }
}

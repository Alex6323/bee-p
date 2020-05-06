use tiny_keccak::{Hasher, Keccak};

use bee_ternary::{
    bigint::{
        common::{BigEndian, U8Repr},
        I384, T242, T243,
    },
    Btrit, Trits, T1B1,
};

use crate::Sponge;

const HASH_LEN: usize = 243;

#[derive(Clone)]
pub struct Kerl {
    keccak: Keccak,
    binary_buffer: I384<BigEndian, U8Repr>,
    ternary_buffer: T243<Btrit>,
}

impl Kerl {
    pub fn new() -> Self {
        Self {
            keccak: Keccak::v384(),
            binary_buffer: I384::<BigEndian, U8Repr>::default(),
            ternary_buffer: T243::<Btrit>::default(),
        }
    }
}

impl Default for Kerl {
    fn default() -> Self {
        Kerl::new()
    }
}

#[derive(Debug)]
pub enum Error {
    NotMultipleOfHashLength,
    TernaryBinaryConversion(bee_ternary::bigint::common::Error),
}

impl From<bee_ternary::bigint::common::Error> for Error {
    fn from(error: bee_ternary::bigint::common::Error) -> Self {
        Error::TernaryBinaryConversion(error)
    }
}

impl Sponge for Kerl {
    const IN_LEN: usize = HASH_LEN;
    const OUT_LEN: usize = HASH_LEN;

    type Error = Error;

    /// Absorb `input` into the sponge by copying `HASH_LEN` chunks of it into its internal
    /// state and transforming the state before moving on to the next chunk.
    ///
    /// If `input` is not a multiple of `HASH_LEN` with the last chunk having `n < HASH_LEN` trits,
    /// the last chunk will be copied to the first `n` slots of the internal state. The remaining
    /// data in the internal state is then just the result of the last transformation before the
    /// data was copied, and will be reused for the next transformation.
    fn absorb(&mut self, input: &Trits) -> Result<(), Self::Error> {
        if input.len() % Self::IN_LEN != 0 {
            return Err(Error::NotMultipleOfHashLength);
        }

        for trits_chunk in input.chunks(Self::IN_LEN) {
            self.ternary_buffer.inner_mut().copy_from(&trits_chunk);
            // Unwrapping is ok because this cannot fail.
            //
            // TODO: Replace with a dedicated `TryFrom` implementation with `Error = !`.
            //
            // TODO: Convert to `t242` without cloning.
            //
            // TODO: Convert to binary without cloning.
            self.binary_buffer = self.ternary_buffer.clone().into_t242().into();

            self.keccak.update(self.binary_buffer.inner_ref());
        }

        Ok(())
    }

    /// Reset the internal state by overwriting it with zeros.
    fn reset(&mut self) {
        // TODO: Overwrite the internal buffer directly rather then setting it to a new Keccak
        // object. This requires using `KeccakState::reset` via a new method `Keccak::method`
        // calling its internal state.
        self.keccak = Keccak::v384();
    }

    /// Squeeze the sponge by copying the calculated hash into the provided `buf`. This will fill
    /// the buffer in chunks of `HASH_LEN` at a time.
    ///
    /// If the last chunk is smaller than `HASH_LEN`, then only the fraction that fits is written
    /// into it.
    fn squeeze_into(&mut self, buf: &mut Trits<T1B1>) -> Result<(), Self::Error> {
        if buf.len() % Self::OUT_LEN != 0 {
            return Err(Error::NotMultipleOfHashLength);
        }

        for trit_chunk in buf.chunks_mut(Self::OUT_LEN) {
            // Create a new Keccak in lieu of resetting the internal one
            let mut keccak = Keccak::v384();

            // Swap out the internal one and the new one
            std::mem::swap(&mut self.keccak, &mut keccak);

            keccak.finalize(&mut self.binary_buffer.inner_mut()[..]);
            let ternary_value = T242::from_i384_ignoring_mst(self.binary_buffer).into_t243();

            trit_chunk.copy_from(&ternary_value.inner_ref());
            self.binary_buffer.not_inplace();
            self.keccak.update(self.binary_buffer.inner_ref());
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bee_ternary::{T1B1Buf, T3B1Buf, TritBuf, TryteBuf};

    macro_rules! test_kerl {
        ($test_name:ident, $input_trytes:expr, $output_trytes:expr) => {
            #[test]
            fn $test_name() {
                let input = $input_trytes;
                let output = $output_trytes;

                let mut kerl = Kerl::new();

                let input_trytes = TryteBuf::try_from_str(input);
                assert!(input_trytes.is_ok());
                let input_trytes = input_trytes.unwrap();

                let input_trit_buf = input_trytes.as_trits().encode::<T1B1Buf>();

                let expected_hash = TryteBuf::try_from_str(output);
                assert!(expected_hash.is_ok());
                let expected_hash = expected_hash.unwrap();

                assert!(kerl.absorb(input_trit_buf.as_slice()).is_ok());

                let output_len = expected_hash
                    .as_trits()
                    .len();
                let mut calculated_hash = TritBuf::<T1B1Buf>::zeros(output_len);
                assert!(kerl.squeeze_into(&mut calculated_hash.as_slice_mut()).is_ok());

                let calculated_hash = calculated_hash.encode::<T3B1Buf>();

                assert_eq!(calculated_hash.as_slice(), expected_hash.as_trits());
            }
        };

        ( $( $test_name:ident: $input_trytes:expr => $output_trytes:expr ),+ $(,)?) => {
            $(
                test_kerl!($test_name, $input_trytes, $output_trytes);
            )+
        }
    }

    test_kerl!(
        from_iota_go_normal_trytes_1:
        "HHPELNTNJIOKLYDUW9NDULWPHCWFRPTDIUWLYUHQWWJVPAKKGKOAZFJPQJBLNDPALCVXGJLRBFSHATF9C"
        =>
        "DMJWZTDJTASXZTHZFXFZXWMNFHRTKWFUPCQJXEBJCLRZOM9LPVJSTCLFLTQTDGMLVUHOVJHBBUYFD9AXX",
        from_iota_go_normal_trytes_2:
        "QAUGQZQKRAW9GKEFIBUD9BMJQOABXBTFELCT9GVSZCPTZOSFBSHPQRWJLLWURPXKNAOWCSVWUBNDSWMPW"
        =>
        "HOVOHFEPCIGTOFEAZVXAHQRFFRTPQEEKANKFKIHUKSGRICVADWDMBINDYKRCCIWBEOPXXIKMLNSOHEAQZ",
        from_iota_go_normal_trytes_3:
        "MWBLYBSRKEKLDHUSRDSDYZRNV9DDCPN9KENGXIYTLDWPJPKBHQBOALSDH9LEJVACJAKJYPCFTJEROARRW"
        =>
        "KXBKXQUZBYZFSYSPDPCNILVUSXOEHQWWWFKZPFCQ9ABGIIQBNLSWLPIMV9LYNQDDYUS9L9GNUIYKYAGVZ",
        from_iota_go_output_with_non_zero_243rd_trit:
        "GYOMKVTSNHVJNCNFBBAH9AAMXLPLLLROQY99QN9DLSJUHDPBLCFFAIQXZA9BKMBJCYSFHFPXAHDWZFEIZ"
        =>
        "OXJCNFHUNAHWDLKKPELTBFUCVW9KLXKOGWERKTJXQMXTKFKNWNNXYD9DMJJABSEIONOSJTTEVKVDQEWTW",
        from_iota_go_input_with_243_trits:
        "EMIDYNHBWMBCXVDEFOFWINXTERALUKYYPPHKP9JJFGJEIUY9MUDVNFZHMMWZUYUSWAIOWEVTHNWMHANBH"
        =>
        "EJEAOOZYSAWFPZQESYDHZCGYNSTWXUMVJOVDWUNZJXDGWCLUFGIMZRMGCAZGKNPLBRLGUNYWKLJTYEAQX",
        from_iota_go_output_with_more_than_243_trits:
        "9MIDYNHBWMBCXVDEFOFWINXTERALUKYYPPHKP9JJFGJEIUY9MUDVNFZHMMWZUYUSWAIOWEVTHNWMHANBH"
        =>
        "G9JYBOMPUXHYHKSNRNMMSSZCSHOFYOYNZRSZMAAYWDYEIMVVOGKPJBVBM9TDPULSFUNMTVXRKFIDOHUXXVYDLFSZYZTWQYTE9SPYYWYTXJYQ9IFGYOLZXWZBKWZN9QOOTBQMWMUBLEWUEEASRHRTNIQWJQNDWRYLCA",
        from_iota_go_input_and_output_with_more_than_243_trits:
        "G9JYBOMPUXHYHKSNRNMMSSZCSHOFYOYNZRSZMAAYWDYEIMVVOGKPJBVBM9TDPULSFUNMTVXRKFIDOHUXXVYDLFSZYZTWQYTE9SPYYWYTXJYQ9IFGYOLZXWZBKWZN9QOOTBQMWMUBLEWUEEASRHRTNIQWJQNDWRYLCA"
        =>
        "LUCKQVACOGBFYSPPVSSOXJEKNSQQRQKPZC9NXFSMQNRQCGGUL9OHVVKBDSKEQEBKXRNUJSRXYVHJTXBPDWQGNSCDCBAIRHAQCOWZEBSNHIJIGPZQITIBJQ9LNTDIBTCQ9EUWKHFLGFUVGGUWJONK9GBCDUIMAYMMQX",
        negative_byte_input:
        "DJ9WGAKRZOMH9KVRCHGCDCREXZVDKY9FXAXVSLELYADXHQCQQSMQYAEEBTEIWTQDUZIOFSFLBQQA9RUPX"
        =>
        "XRZCRWFXU9UYRKFQRKWROIRGEVGTUGUBKDYGPWDTUXXOFVXWRTQBRRGGUSIEMPAISTUEYEZJXXEPUTY9D",
    );
}

use crypto::{Sponge, Trits, TritsBuf};
// TODO Remove when available in bee
use crate::Seed;
use iota_conversion::Trinary;
use rand::Rng;
use std::marker::PhantomData;

// TODO Put constants in a separate file

// TODO: documentation
pub const MIN_TRIT_VALUE: i8 = -1;
// TODO: documentation
pub const MAX_TRIT_VALUE: i8 = 1;
// TODO: documentation
pub const TRYTE_ALPHABET: &[u8] = b"9ABCDEFGHIJKLMNOPQRSTUVWXYZ";

// TODO: documentation
pub struct IotaSeed<S> {
    seed: TritsBuf,
    _sponge: PhantomData<S>,
}

// TODO: documentation
#[derive(Debug, PartialEq)]
pub enum IotaSeedError {
    InvalidLength(usize),
    InvalidTrit(i8),
}

impl<S: Sponge + Default> Seed for IotaSeed<S> {
    type Error = IotaSeedError;
    // TODO: documentation
    // TODO: is this random enough ?
    fn new() -> Self {
        let mut rng = rand::thread_rng();
        let seed: String = (0..81)
            .map(|_| TRYTE_ALPHABET[rng.gen_range(0, TRYTE_ALPHABET.len())] as char)
            .collect();

        Self {
            seed: TritsBuf::from_i8_unchecked(seed.trits()),
            _sponge: PhantomData,
        }
    }

    // TODO: documentation
    fn subseed(&self, index: u64) -> Self {
        let mut sponge = S::default();
        let mut subseed = self.seed.clone();

        // TODO Put in trit utilities file
        for _ in 0..index {
            for trit in subseed.inner_mut().iter_mut() {
                *trit += 1;
                if *trit > MAX_TRIT_VALUE {
                    *trit = MIN_TRIT_VALUE;
                } else {
                    break;
                }
            }
        }

        let tmp = match sponge.digest(&subseed.as_trits()) {
            Ok(buf) => buf,
            Err(_) => unreachable!(),
        };

        Self {
            seed: tmp,
            _sponge: PhantomData,
        }
    }

    // TODO: documentation
    fn from_bytes(bytes: &[i8]) -> Result<Self, Self::Error> {
        if bytes.len() != 243 {
            return Err(Self::Error::InvalidLength(bytes.len()));
        }

        for byte in bytes {
            match byte {
                -1 | 0 | 1 => continue,
                _ => return Err(Self::Error::InvalidTrit(*byte)),
            }
        }

        Ok(Self {
            seed: TritsBuf::from_i8_unchecked(bytes),
            _sponge: PhantomData,
        })
    }

    // TODO: documentation
    fn to_bytes(&self) -> &[i8] {
        // &self.0.to_bytes()
        self.seed.inner_ref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::slice_eq;
    use crypto::{CurlP27, CurlP81};

    const IOTA_SEED: &str =
        "ABCDEFGHIJKLMNOPQRSTUVWXYZ9ABCDEFGHIJKLMNOPQRSTUVWXYZ9ABCDEFGHIJKLMNOPQRSTUVWXYZ9";

    #[test]
    fn iota_seed_new_test() {
        for _ in 0..10 {
            let iota_seed = IotaSeed::<CurlP27>::new();

            for byte in iota_seed.to_bytes() {
                assert!(*byte == -1 || *byte == 0 || *byte == 1);
            }
        }
    }

    fn iota_seed_subseed_generic_test<S: Sponge + Default>(
        iota_seed_string: &str,
        iota_subseed_strings: &[&str],
    ) {
        let iota_seed = IotaSeed::<S>::from_bytes(&iota_seed_string.trits()).unwrap();

        for (i, iota_subseed_string) in iota_subseed_strings.iter().enumerate() {
            let iota_subseed = iota_seed.subseed(i as u64);

            assert!(slice_eq(
                iota_subseed.to_bytes(),
                &iota_subseed_string.trits()
            ));
        }
    }

    #[test]
    fn iota_seed_subseed_curl27_test() {
        iota_seed_subseed_generic_test::<CurlP27>(
            IOTA_SEED,
            &[
                "ITTFAEIWTRSFQGZGLGUMLUTHFXYSCLXTFYMGVTTDSNNWFUCKBRPSOBERNLXIYCNCEBKUV9QIXI9BDCKSM",
                "W9YWLOQQJMENWCDBLBKYBNJJDGFKFBGYEBSIBPKUAGNIV9TJWRRAQPAEKBLIYVLGHPIIDYQYP9QNSPFTY",
                "X9WMLHFSJYEWNLVSGTVGWMAPNUSFMXQPTMCPUML9RCMAJQVUYMTJJHKT9HO9NSNGAEMKGDBHE9KZNMBPZ",
                "YNTUYQNJWJPK99YE9NOMGNKF9YRBJX9EH9UZWLMISXQRQLLZRKHFOPTW9PIERIPXK9ZDUPLSLZOEFUWXF",
                "URBRFVWBAGHM9WTWSZZLRBMNGMNNRJRBGBLDEBBSZTGMWELW9JHXFSFNLRKPI9MLYELEZEDYIPKGE9CRO",
                "XMGTGBZBINHC9ZPKRBHZFLUP9CEWULNCMVUAVVUXRDHU9OILDOORKPLRIWZQDNRFGSWMJAVYZWGDXMZNW",
                "KFEGWPGWLAHWQXGCHKHDDVAZEISLYMGQLRRZBCJWXWKK9JIJKHXRDV9NMYIFTAGKXU9GLACAQUCXBLMH9",
                "BMUAOOZBHPUOVHRWPX9KWUCZSXWXWPMKOMGNAZOXLDMAHBBVMDLXQ9IVPOPIOFPWHZSMRKBOBLCUEVUXX",
                "GLVXLLOFYERJWBECYRXVPCFXK9GUDCHBEZYMTPMUDOYEQCIAPCAACKSOL9ADEGSTBQRIBJIWTCJYVUIRW",
                "FOPHLVKCYHZLLCCOUWBPMQQAWHVRBGJBKQGPQXOTOEWTOCVZQCJXDCBLG9SEZBUVYPIIRTTP9CJPXWKKW",
            ],
        );
    }

    #[test]
    fn iota_seed_subseed_curl81_test() {
        iota_seed_subseed_generic_test::<CurlP81>(
            IOTA_SEED,
            &[
                "PKKJZREHPYHNIBWAPYEXHXEAFZCI99UWZNKBOCCECFTDUXG9YGYDAGRLUBJVKMYNWPRCPYENACHOYSHJO",
                "EM9CGOOPJNDODXNHATOQTKLPV9SCMMDHMZIBQUZJCUBCPVAGP9AIEAKYAXOYTEUXRKZACVXRHGWNW9TNC",
                "RRJNNVVOJEGYSXWUDUBVZSYSSWXLIAYUPIEAFSWUDDDEFCTRBBTMODUSXASEONBJOAREKLARUOUDHWKZF",
                "XNW9XBGHM9ZVPSV9BXMFRB9MKODAXKEPPSTGX9PFEDNTVZPJUQGGQ9JCOZRMABQQNQBAURFKVJUZTYUQV",
                "MMJRVEANOJUYWEGF9NNJUJVVZTGXKRWGXGVXRNRNDHPNMWVDGRHRH9FGODYVYWSVABUYZEVCJXUZZLYQB",
                "PCOAKZFKIWGDTTQSBWZABUCIIEFADQQFHCJYTOFVEURSEQZHQCORMMBDKVRGNATYINDDWMGZBUGKLUZOR",
                "CMDZYS9GCHCFFOHPMIPDKRASMFSUXJPDWUWYNMHLHBXUPUPPLEKCSBWSKUG9TKTCRXHJHIA9BVWKAGEHG",
                "TAIMONWQMIXTMCGYMBGIDOZF9FOUPBIEIYYPQZYNMORHGNNLAPWCSMAKVLREZLGDS9XGTXNYYYQYUWRPM",
                "VTKERDSFSJGLZF9UJHXJKFXIXFYSPNVSBHBMAZXXCJCBJHLDEEDMNPBRFJ9PCLNNSZYFLMRJQAYRMHVWL",
                "YVGEVYOLICOIDRYBHP99JQZZJKVYZDPHFCQKJAN9BCEZCMWIEUJIRZWNAZNUMNDMT9JUCDGBSGXDUYQJC",
            ],
        );
    }

    // #[test]
    // fn iota_seed_subseed_kerl_test() {
    //     iota_seed_subseed_generic_test::<Kerl>(
    //         IOTA_SEED,
    //         &[
    //             "APSNZAPLANAGSXGZMZYCSXROJ9KUX9HVOPODQHMWNJOCGBKRIOOQKYGPFAIQBYNIODMIWMFKJGKRWFFPY",
    //             "PXQMW9VMXGYTEPYPIASGPQ9CAQUQWNSUIIVHFIEAB9C9DHNNCWSNJKSBEAKYIBCYOZDDTQANEKPGJPVIY",
    //             "ZUJWIFUVFGOGDNMTFDVZGTWVCBVIK9XQQDQEKJSKBXNGLFLLIPTVUHHPCPKNMBFMATPYJVOH9QTEVOYTW",
    //             "OCHUZGFIX9VXXMBJXPKAPZHXIOCLAEKREMCKQIYQPXQQLRTOEUQRCZIYVSLUTJQGISGDRDSCERBOEEI9C",
    //             "GWTMVQWHHCYFXVHGUYYZHUNXICJLMSOZVBAZOIZIWGBRAXMFDUBLP9NVIFEFFRARYIHNGPEBLNUECABKW",
    //             "XWIYCHCVZEXOPXCQEJUGPMGVAIYBULVHWDD9YWMAZNJQEISHOBMYFHZKCBT9GWCSRQSFURKF9I9ITWEUC",
    //             "XRBHXHE9IVEDFHQPNNMYOPXOLPXRBSYCGQNMRFKYENRJZLZAVMFLUCWWCNBFPKOSHF9UPMFFEWAWAHJP9",
    //             "IP9DGBVAPNHHDP9CXOBYRLTYVJCQYUUWNWGNFUSDRKFIIAVPYPQDASDULPJBBEBOQATDHV9PVXYIJFQTA",
    //             "XSGWTBAECBMTKEHXNYAVSYRPLASPJSHPIWROHRLDFUEKISEMCMXYGRZMPZCEAKZ9UKQBA9LEQFXWEMZPD",
    //             "JXCAHDZVVCMGIGWJFFVDRFCHKBVAWTSLWIPZYGBECFXJQPDNDYJTEYCBHSRPDMPFEPWZUMDEIPIBW9SI9",
    //         ],
    //     );
    // }

    #[test]
    fn iota_seed_from_bytes_invalid_length_test() {
        let iota_seed_bytes = [0; 42];

        match IotaSeed::<CurlP27>::from_bytes(&iota_seed_bytes) {
            Err(IotaSeedError::InvalidLength(len)) => assert_eq!(len, 42),
            _ => unreachable!(),
        }
    }

    #[test]
    fn iota_seed_from_bytes_invalid_trit_test() {
        let iota_seed_bytes = &mut IOTA_SEED.trits();

        iota_seed_bytes[100] = 42;

        match IotaSeed::<CurlP27>::from_bytes(&iota_seed_bytes) {
            Err(IotaSeedError::InvalidTrit(byte)) => assert_eq!(byte, 42),
            _ => unreachable!(),
        }
    }

    #[test]
    fn iota_seed_to_bytes_from_bytes_test() {
        for _ in 0..10 {
            let iota_seed_1 = IotaSeed::<CurlP27>::new();
            let iota_seed_2 = IotaSeed::<CurlP27>::from_bytes(iota_seed_1.to_bytes()).unwrap();

            assert!(slice_eq(iota_seed_1.to_bytes(), iota_seed_2.to_bytes()));
        }
    }
}

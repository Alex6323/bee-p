use crate::Seed;

use bee_crypto::Sponge;
use bee_ternary::{
    Btrit,
    Trit,
    TritBuf,
    Trits,
    TRYTE_ALPHABET,
};

use rand::Rng;
use std::marker::PhantomData;

// TODO Put constants in a separate file

// TODO: documentation
pub const MIN_TRIT_VALUE: i8 = -1;
// TODO: documentation
pub const MAX_TRIT_VALUE: i8 = 1;

// TODO: documentation
pub struct IotaSeed<S> {
    seed: TritBuf,
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
        // TODO out of here ?
        let trits = [-1, 0, 1];
        let seed: Vec<i8> = (0..243).map(|_| trits[rng.gen_range(0, trits.len())]).collect();

        Self {
            seed: TritBuf::from_i8_unchecked(&seed),
            _sponge: PhantomData,
        }
    }

    // TODO: documentation
    fn from_buf(buf: TritBuf) -> Result<Self, Self::Error> {
        if buf.len() != 243 {
            Err(Self::Error::InvalidLength(buf.len()))?;
        }

        Ok(Self {
            seed: buf,
            _sponge: PhantomData,
        })
    }

    // TODO: documentation
    fn as_bytes(&self) -> &[i8] {
        self.seed.as_i8_slice()
    }

    fn trits(&self) -> &Trits {
        &self.seed
    }
}

impl<S: Sponge + Default> IotaSeed<S> {
    // TODO: documentation
    pub fn subseed(&self, index: u64) -> Self {
        let mut sponge = S::default();
        let mut subseed = self.seed.clone();

        for _ in 0..index {
            // TODO Put in trit utilities file
            for i in 0..subseed.len() {
                if let Some(ntrit) = subseed.get(i).unwrap().checked_increment() {
                    subseed.set(i, ntrit);
                    break;
                } else {
                    subseed.set(i, Btrit::NegOne);
                }
            }
        }

        // TODO return error
        let tmp = match sponge.digest(&subseed) {
            Ok(buf) => buf,
            Err(_) => unreachable!(),
        };

        Self {
            seed: tmp,
            _sponge: PhantomData,
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    use bee_crypto::{
        CurlP27,
        CurlP81,
        Kerl,
    };
    use bee_ternary::{
        T1B1Buf,
        TryteBuf,
    };

    const IOTA_SEED: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZ9ABCDEFGHIJKLMNOPQRSTUVWXYZ9ABCDEFGHIJKLMNOPQRSTUVWXYZ9";

    #[test]
    fn iota_seed_new_test() {
        for _ in 0..10 {
            let iota_seed = IotaSeed::<CurlP27>::new();
            for byte in iota_seed.as_bytes() {
                assert!(*byte == -1 || *byte == 0 || *byte == 1);
            }
        }
    }

    fn iota_seed_subseed_generic_test<S: Sponge + Default>(iota_seed_string: &str, iota_subseed_strings: &[&str]) {
        let iota_seed_trits = TryteBuf::try_from_str(iota_seed_string)
            .unwrap()
            .as_trits()
            .encode::<T1B1Buf>();
        let iota_seed = IotaSeed::<S>::from_buf(iota_seed_trits).unwrap();

        for (i, iota_subseed_string) in iota_subseed_strings.iter().enumerate() {
            let iota_subseed = iota_seed.subseed(i as u64);
            let iota_subseed_trits = TryteBuf::try_from_str(iota_subseed_string)
                .unwrap()
                .as_trits()
                .encode::<T1B1Buf>();

            assert_eq!(iota_subseed.as_bytes(), iota_subseed_trits.as_i8_slice());
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

        match IotaSeed::<CurlP27>::from_buf(TritBuf::from_i8_unchecked(&iota_seed_bytes)) {
            Err(IotaSeedError::InvalidLength(len)) => assert_eq!(len, 42),
            _ => unreachable!(),
        }
    }

    // #[test]
    // fn iota_seed_from_bytes_invalid_trit_test() {
    //     let iota_seed_bytes = &mut IOTA_SEED.trits();
    //
    //     iota_seed_bytes[100] = 42;
    //
    //     match IotaSeed::<CurlP27>::from_buf(TritBuf::from_i8_unchecked(&iota_seed_bytes)) {
    //         Err(IotaSeedError::InvalidTrit(byte)) => assert_eq!(byte, 42),
    //         _ => unreachable!(),
    //     }
    // }

    #[test]
    fn iota_seed_to_bytes_from_bytes_test() {
        for _ in 0..10 {
            let iota_seed_1 = IotaSeed::<CurlP27>::new();
            let iota_seed_2 = IotaSeed::<CurlP27>::from_buf(iota_seed_1.trits().to_buf()).unwrap();

            assert_eq!(iota_seed_1.as_bytes(), iota_seed_2.as_bytes());
        }
    }
}

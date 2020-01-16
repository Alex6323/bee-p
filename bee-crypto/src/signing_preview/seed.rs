// TODO Replace with bee impl when available
use iota_crypto::Sponge;
// TODO Remove when available in bee
use iota_conversion::Trinary;
use rand::Rng;

// TODO Put constants in a separate file

// TODO: documentation
pub const MIN_TRIT_VALUE: i8 = -1;
// TODO: documentation
pub const MAX_TRIT_VALUE: i8 = 1;
// TODO: documentation
pub const TRYTE_ALPHABET: &[u8] = b"9ABCDEFGHIJKLMNOPQRSTUVWXYZ";

// TODO: documentation
pub struct Seed([i8; 243]);

// TODO: documentation
#[derive(Debug, PartialEq)]
pub enum SeedError {
    InvalidLength,
    InvalidTrit,
}

// TODO: documentation
impl Seed {
    // TODO: documentation
    // TODO: is this random enough ?
    pub fn new() -> Self {
        let mut rng = rand::thread_rng();
        let seed: String = (0..81)
            .map(|_| TRYTE_ALPHABET[rng.gen_range(0, TRYTE_ALPHABET.len())] as char)
            .collect();

        Self::from_bytes_unchecked(&seed.trits())
    }

    // TODO: documentation
    pub fn subseed<S: Sponge + Default>(&self, index: u64) -> Self {
        let mut sponge = S::default();
        let mut subseed = self.0;

        // TODO Put in trit utilities file
        for _ in 0..index {
            for trit in subseed.iter_mut() {
                *trit += 1;
                if *trit > MAX_TRIT_VALUE {
                    *trit = MIN_TRIT_VALUE;
                } else {
                    break;
                }
            }
        }

        sponge.absorb(&subseed).unwrap();
        sponge.squeeze(&mut subseed).unwrap();
        sponge.reset();

        Self::from_bytes_unchecked(&subseed)
    }

    // TODO: documentation
    pub fn from_bytes(bytes: &[i8]) -> Result<Self, SeedError> {
        if bytes.len() != 243 {
            return Err(SeedError::InvalidLength);
        }

        for byte in bytes {
            match byte {
                -1 | 0 | 1 => continue,
                _ => return Err(SeedError::InvalidTrit),
            }
        }

        Ok(Self::from_bytes_unchecked(bytes))
    }

    // TODO: documentation
    fn from_bytes_unchecked(bytes: &[i8]) -> Self {
        let mut seed = [0; 243];

        seed.copy_from_slice(bytes);

        Seed(seed)
    }

    // TODO: documentation
    pub fn to_bytes(&self) -> &[i8] {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    // TODO super::super ?
    use super::super::slice_eq;
    // TODO Remove when available in bee
    use iota_crypto::{Curl, Kerl};

    const SEED: &str =
        "ABCDEFGHIJKLMNOPQRSTUVWXYZ9ABCDEFGHIJKLMNOPQRSTUVWXYZ9ABCDEFGHIJKLMNOPQRSTUVWXYZ9";

    #[test]
    fn seed_new_test() {
        for _ in 0..10 {
            let seed = Seed::new();

            for byte in seed.to_bytes() {
                assert_eq!(*byte == -1 || *byte == 0 || *byte == 1, true);
            }
        }
    }

    fn seed_subseed_generic_test<S: Sponge + Default>(seed_string: &str, subseed_strings: &[&str]) {
        let seed = Seed::from_bytes(&seed_string.trits()).unwrap();

        for (i, subseed_string) in subseed_strings.iter().enumerate() {
            let subseed = seed.subseed::<S>(i as u64);

            assert!(slice_eq(subseed.to_bytes(), &subseed_string.trits()));
        }
    }

    #[test]
    fn seed_subseed_curl81_test() {
        seed_subseed_generic_test::<Curl>(
            SEED,
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

    // TODO Will be activated when Curl27 is a proper type
    // #[test]
    // fn seed_subseed_curl27_test() {
    //     seed_subseed_generic_test::<Curl>(
    //         SEED,
    //         &[
    //             "ITTFAEIWTRSFQGZGLGUMLUTHFXYSCLXTFYMGVTTDSNNWFUCKBRPSOBERNLXIYCNCEBKUV9QIXI9BDCKSM",
    //             "W9YWLOQQJMENWCDBLBKYBNJJDGFKFBGYEBSIBPKUAGNIV9TJWRRAQPAEKBLIYVLGHPIIDYQYP9QNSPFTY",
    //             "X9WMLHFSJYEWNLVSGTVGWMAPNUSFMXQPTMCPUML9RCMAJQVUYMTJJHKT9HO9NSNGAEMKGDBHE9KZNMBPZ",
    //             "YNTUYQNJWJPK99YE9NOMGNKF9YRBJX9EH9UZWLMISXQRQLLZRKHFOPTW9PIERIPXK9ZDUPLSLZOEFUWXF",
    //             "URBRFVWBAGHM9WTWSZZLRBMNGMNNRJRBGBLDEBBSZTGMWELW9JHXFSFNLRKPI9MLYELEZEDYIPKGE9CRO",
    //             "XMGTGBZBINHC9ZPKRBHZFLUP9CEWULNCMVUAVVUXRDHU9OILDOORKPLRIWZQDNRFGSWMJAVYZWGDXMZNW",
    //             "KFEGWPGWLAHWQXGCHKHDDVAZEISLYMGQLRRZBCJWXWKK9JIJKHXRDV9NMYIFTAGKXU9GLACAQUCXBLMH9",
    //             "BMUAOOZBHPUOVHRWPX9KWUCZSXWXWPMKOMGNAZOXLDMAHBBVMDLXQ9IVPOPIOFPWHZSMRKBOBLCUEVUXX",
    //             "GLVXLLOFYERJWBECYRXVPCFXK9GUDCHBEZYMTPMUDOYEQCIAPCAACKSOL9ADEGSTBQRIBJIWTCJYVUIRW",
    //             "FOPHLVKCYHZLLCCOUWBPMQQAWHVRBGJBKQGPQXOTOEWTOCVZQCJXDCBLG9SEZBUVYPIIRTTP9CJPXWKKW",
    //         ],
    //     );
    // }

    #[test]
    fn seed_subseed_kerl_test() {
        seed_subseed_generic_test::<Kerl>(
            SEED,
            &[
                "APSNZAPLANAGSXGZMZYCSXROJ9KUX9HVOPODQHMWNJOCGBKRIOOQKYGPFAIQBYNIODMIWMFKJGKRWFFPY",
                "PXQMW9VMXGYTEPYPIASGPQ9CAQUQWNSUIIVHFIEAB9C9DHNNCWSNJKSBEAKYIBCYOZDDTQANEKPGJPVIY",
                "ZUJWIFUVFGOGDNMTFDVZGTWVCBVIK9XQQDQEKJSKBXNGLFLLIPTVUHHPCPKNMBFMATPYJVOH9QTEVOYTW",
                "OCHUZGFIX9VXXMBJXPKAPZHXIOCLAEKREMCKQIYQPXQQLRTOEUQRCZIYVSLUTJQGISGDRDSCERBOEEI9C",
                "GWTMVQWHHCYFXVHGUYYZHUNXICJLMSOZVBAZOIZIWGBRAXMFDUBLP9NVIFEFFRARYIHNGPEBLNUECABKW",
                "XWIYCHCVZEXOPXCQEJUGPMGVAIYBULVHWDD9YWMAZNJQEISHOBMYFHZKCBT9GWCSRQSFURKF9I9ITWEUC",
                "XRBHXHE9IVEDFHQPNNMYOPXOLPXRBSYCGQNMRFKYENRJZLZAVMFLUCWWCNBFPKOSHF9UPMFFEWAWAHJP9",
                "IP9DGBVAPNHHDP9CXOBYRLTYVJCQYUUWNWGNFUSDRKFIIAVPYPQDASDULPJBBEBOQATDHV9PVXYIJFQTA",
                "XSGWTBAECBMTKEHXNYAVSYRPLASPJSHPIWROHRLDFUEKISEMCMXYGRZMPZCEAKZ9UKQBA9LEQFXWEMZPD",
                "JXCAHDZVVCMGIGWJFFVDRFCHKBVAWTSLWIPZYGBECFXJQPDNDYJTEYCBHSRPDMPFEPWZUMDEIPIBW9SI9",
            ],
        );
    }

    #[test]
    fn seed_from_bytes_invalid_length_test() {
        let seed_bytes = [0; 42];

        match Seed::from_bytes(&seed_bytes) {
            Ok(_) => unreachable!(),
            Err(err) => assert_eq!(err, SeedError::InvalidLength),
        }
    }

    #[test]
    fn seed_from_bytes_invalid_trit_test() {
        let seed_bytes = &mut SEED.trits();

        seed_bytes[100] = 42;

        match Seed::from_bytes(&seed_bytes) {
            Ok(_) => unreachable!(),
            Err(err) => assert_eq!(err, SeedError::InvalidTrit),
        }
    }

    #[test]
    fn seed_to_bytes_from_bytes_test() {
        for _ in 0..10 {
            let seed_1 = Seed::new();
            let seed_2 = Seed::from_bytes(seed_1.to_bytes()).unwrap();

            assert!(slice_eq(seed_1.to_bytes(), seed_2.to_bytes()));
        }
    }
}

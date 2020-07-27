// Copyright 2020 IOTA Stiftung
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file except in compliance with
// the License. You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software distributed under the License is distributed on
// an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and limitations under the License.

use bee_crypto::ternary::sponge::{Kerl, Sponge};
use bee_signing::ternary::seed::{Error, Seed};
use bee_ternary::{Btrit, T1B1Buf, TritBuf, TryteBuf};

const IOTA_SEED: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZ9ABCDEFGHIJKLMNOPQRSTUVWXYZ9ABCDEFGHIJKLMNOPQRSTUVWXYZ9";

#[test]
fn new() {
    for _ in 0..10 {
        let iota_seed = Seed::new();
        for byte in iota_seed.as_trits().iter() {
            assert!(byte == Btrit::NegOne || byte == Btrit::Zero || byte == Btrit::PlusOne);
        }
    }
}

fn subseed_generic<S: Sponge + Default>(iota_seed_string: &str, iota_subseed_strings: &[&str]) {
    let iota_seed_trits = TryteBuf::try_from_str(iota_seed_string)
        .unwrap()
        .as_trits()
        .encode::<T1B1Buf>();
    let iota_seed = Seed::from_trits(iota_seed_trits).unwrap();

    for (i, iota_subseed_string) in iota_subseed_strings.iter().enumerate() {
        let iota_subseed = iota_seed.subseed(i as u64);
        let iota_subseed_trits = TryteBuf::try_from_str(iota_subseed_string)
            .unwrap()
            .as_trits()
            .encode::<T1B1Buf>();

        assert_eq!(iota_subseed.as_trits(), iota_subseed_trits.as_slice());
    }
}

#[test]
fn subseed_kerl() {
    subseed_generic::<Kerl>(
        IOTA_SEED,
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
fn iota_seed_from_bytes_invalid_length() {
    let buf = TritBuf::zeros(42);

    match Seed::from_trits(buf) {
        Err(Error::InvalidLength(len)) => assert_eq!(len, 42),
        _ => unreachable!(),
    }
}

#[test]
fn iota_seed_to_bytes_from_bytes() {
    for _ in 0..10 {
        let iota_seed_1 = Seed::new();
        let iota_seed_2 = Seed::from_trits(iota_seed_1.as_trits().to_buf()).unwrap();

        assert_eq!(iota_seed_1.as_trits(), iota_seed_2.as_trits());
    }
}

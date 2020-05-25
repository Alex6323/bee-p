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

use bee_ternary::T5B1Buf;
use bee_transaction::{BundledTransactionField, Hash};

use std::marker::PhantomData;

use bytemuck::cast_slice;
use digest::Digest;

const LEAF_PREFIX: u8 = 0x00;
const NODE_PREFIX: u8 = 0x01;

#[derive(Default)]
pub(crate) struct Merkle<H: Default + Digest> {
    hasher: PhantomData<H>,
}

impl<H: Default + Digest> Merkle<H> {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    fn empty(&mut self) -> Vec<u8> {
        (&H::digest(b"")).to_vec()
    }

    fn leaf(&mut self, hash: Hash) -> Vec<u8> {
        let mut hasher = H::default();

        hasher.input([LEAF_PREFIX]);
        hasher.input(cast_slice(hash.to_inner().encode::<T5B1Buf>().as_i8_slice()));
        (&hasher.result_reset()).to_vec()
    }

    fn node(&mut self, hashes: &[Hash]) -> Vec<u8> {
        let mut hasher = H::default();
        let n = hashes.len() as u32 - 1;
        let k = 1 << (32 - n.leading_zeros() - 1);

        hasher.input([NODE_PREFIX]);
        hasher.input(self.hash(&hashes[0..k]));
        hasher.input(self.hash(&hashes[k..]));
        (&hasher.result_reset()).to_vec()
    }

    pub(crate) fn hash(&mut self, hashes: &[Hash]) -> Vec<u8> {
        match hashes.len() {
            0 => self.empty(),
            1 => self.leaf(hashes[0]),
            _ => self.node(hashes),
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    use bee_ternary::{T1B1Buf, TryteBuf};

    use blake2::Blake2b;

    #[test]
    fn empty() {
        let hash = Merkle::<Blake2b>::new().hash(&[]);

        assert_eq!(
            hex::encode(hash),
            "786a02f742015903c6c6fd852552d272912f4740e15847618a86e217f71f5419d25e1031afee585313896444934eb04b903a685b14\
            48b755d56f701afe9be2ce"
        )
    }

    #[test]
    fn null_leaf() {
        let hash = Merkle::<Blake2b>::new().hash(&vec![Hash::zeros()]);

        assert_eq!(
            hex::encode(hash),
            "0c18f7cbf23c3c8eda01ab64c79379ff0bf0d854125cbdf7dba43ca7630171d84c042673b731cb9f92cf937d738152306a8db092d9\
            413d531dd8a4299c05278f"
        )
    }

    #[test]
    fn null_node() {
        let hash = Merkle::<Blake2b>::new().hash(&vec![Hash::zeros(), Hash::zeros()]);

        assert_eq!(
            hex::encode(hash),
            "876b38297f865de8b89fa69d7daa4da0fc31f562228ac4b5b71009ec10e878a7aec06f48ddf98a16460b742673ed47f308ff577684\
            26bf72a6aee27d1c4ba5fd"
        )
    }

    #[test]
    fn tree() {
        let mut hashes = Vec::new();

        for hash in [
            "NOBKDFGZMOWYUKDZITTWBRWA9YPSXCVFENCQFPC9GMJIAIPSSURYIOMYZLGNZXLUAQHHNBSRHNOIJDYZO",
            "IPATPTEZSBMFJRDCRPTCVUQWBAVCAXAVZIDEDL9TSILDFWDMIIFPZIYHKRFFZDYQNKBQBVGYSKMLCYBMR",
            "MXOIOFOGLIHCHMDRCWAIYCWIUCMGEZWXFJZFWBRCNSNBWIGFJXBCACPKMLLANYNXSGYKANYFTVGTLFXXX",
            "EXZTJAXJMZJBBIZGUTMBOEUQDNVHJPXCLFUXNLPLSBATDMKYUZOFMHCOBWUABYDMNGMKIXLIUFXNVY9PN",
            "SJXYVFUDCDPPAOALVXDQUKAWLLOQO99OSJQT9TUNILQ9VLFLCZMLZAKUTIZFHOLPMGPYHKMMUUSURIOCF",
            "Q9GHMAITEZCWKFIESJARYQYMF9XWFPQTTFRXULLHQDWEZLYBSFYHSLPXEHBORDDFYZRFYFGDCM9VJKEFR",
            "GMNECTSPSLSPPEITCHBXSN9KZD9OZPVPOET9TVQJDZMFGN9SGPRPMUQARNXUVKMWAFAKLKWBZLWZCTPCP",
        ]
        .iter()
        {
            hashes.push(Hash::from_inner_unchecked(
                TryteBuf::try_from_str(hash).unwrap().as_trits().encode::<T1B1Buf>(),
            ));
        }

        let hash = Merkle::<Blake2b>::new().hash(&hashes);

        assert_eq!(
            hex::encode(hash),
            "d07161bdb535afb7dbb3f5b2fb198ecf715cbd9dfca133d2b48d67b1e11173c6f92bed2f4dca92c36e8d1ef279a0c19ca9e40a113e\
            9f5526090342988f86e53a"
        )
    }
}

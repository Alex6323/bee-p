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

#[cfg(test)]
mod tests {

    use bee_crypto::Kerl;
    use bee_signing::ternary::{
        PrivateKeyGenerator, Seed, TernarySeed, WotsSecurityLevel, WotsShakePrivateKeyGeneratorBuilder,
    };
    use bee_ternary::{T1B1Buf, TryteBuf};

    #[test]
    fn wots_shake() {
        let seed_trits =
            TryteBuf::try_from_str("CEFLDDLMF9TO9ZLLTYXIPVFIJKAOFRIQLGNYIDZCTDYSWMNXPYNGFAKHQDY9ABGGQZHEFTXKWKWZXEIUD")
                .unwrap()
                .as_trits()
                .encode::<T1B1Buf>();
        let seed = TernarySeed::<Kerl>::from_buf(seed_trits).unwrap();
        let private_key_generator = WotsShakePrivateKeyGeneratorBuilder::<Kerl>::default()
            .security_level(WotsSecurityLevel::Medium)
            .build()
            .unwrap();
        let private_key = private_key_generator.generate(&seed, 0).unwrap();
    }
}

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

    use bee_snapshot::global::{Error, GlobalSnapshot};

    #[test]
    fn valid() {
        assert_eq!(
            GlobalSnapshot::from_file("tests/files/global_snapshot_valid.txt")
                .unwrap()
                .state()
                .len(),
            3
        );
    }

    #[test]
    fn file_not_found() {
        assert_eq!(
            GlobalSnapshot::from_file("tests/files/global_snapshot_file_not_found.txt").err(),
            Some(Error::FileNotFound)
        );
    }

    #[test]
    fn missing_semicolon() {
        assert_eq!(
            GlobalSnapshot::from_file("tests/files/global_snapshot_missing_semicolon.txt").err(),
            Some(Error::MissingSemicolon)
        );
    }

    #[test]
    fn extraneous_semicolon() {
        assert_eq!(
            GlobalSnapshot::from_file("tests/files/global_snapshot_extraneous_semicolon.txt").err(),
            Some(Error::ExtraneousSemicolon)
        );
    }

    #[test]
    fn invalid_address_tryte() {
        assert_eq!(
            GlobalSnapshot::from_file("tests/files/global_snapshot_invalid_address_tryte.txt").err(),
            Some(Error::InvalidAddressTryte)
        );
    }

    #[test]
    fn invalid_address_length() {
        assert_eq!(
            GlobalSnapshot::from_file("tests/files/global_snapshot_invalid_address_length.txt").err(),
            Some(Error::InvalidAddressLength)
        );
    }

    #[test]
    fn duplicate_address() {
        assert_eq!(
            GlobalSnapshot::from_file("tests/files/global_snapshot_duplicate_address.txt").err(),
            Some(Error::DuplicateAddress)
        );
    }

    #[test]
    fn invalid_balance() {
        assert_eq!(
            GlobalSnapshot::from_file("tests/files/global_snapshot_invalid_balance.txt").err(),
            Some(Error::InvalidBalance)
        );
    }

    #[test]
    fn negative_balance() {
        assert_eq!(
            GlobalSnapshot::from_file("tests/files/global_snapshot_negative_balance.txt").err(),
            Some(Error::InvalidBalance)
        );
    }

    #[test]
    fn overflow_balance() {
        assert_eq!(
            GlobalSnapshot::from_file("tests/files/global_snapshot_overflow_balance.txt").err(),
            Some(Error::InvalidBalance)
        );
    }

    #[test]
    fn null_balance() {
        assert_eq!(
            GlobalSnapshot::from_file("tests/files/global_snapshot_null_balance.txt").err(),
            Some(Error::NullBalance)
        );
    }

    #[test]
    fn invalid_supply_more() {
        assert_eq!(
            GlobalSnapshot::from_file("tests/files/global_snapshot_invalid_supply_more.txt").err(),
            Some(Error::InvalidSupply)
        );
    }

    #[test]
    fn invalid_supply_less() {
        assert_eq!(
            GlobalSnapshot::from_file("tests/files/global_snapshot_invalid_supply_less.txt").err(),
            Some(Error::InvalidSupply)
        );
    }

    #[test]
    fn additional_whitespaces() {
        assert_eq!(
            GlobalSnapshot::from_file("tests/files/global_snapshot_additional_whitespaces.txt").err(),
            Some(Error::InvalidAddressTryte)
        );
    }

    #[test]
    fn different_newline() {
        assert_eq!(
            GlobalSnapshot::from_file("tests/files/global_snapshot_different_newline.txt")
                .unwrap()
                .state()
                .len(),
            3
        );
    }
}

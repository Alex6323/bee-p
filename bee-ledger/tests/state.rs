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

use bee_ledger::{diff::LedgerDiff, state::LedgerState};
use bee_test::field::rand_trits_field;
use bee_transaction::bundled::Address;

use std::collections::HashMap;

#[test]
fn get_or_zero_single_get() {
    let mut inner_state = HashMap::new();
    let addr = rand_trits_field::<Address>();

    inner_state.insert(addr.clone(), 42);

    let state: LedgerState = inner_state.into();

    assert_eq!(state.get_or_zero(&addr), 42);
}

#[test]
fn get_or_zero_single_zero() {
    let mut inner_state = HashMap::new();
    let addr1 = rand_trits_field::<Address>();
    let addr2 = rand_trits_field::<Address>();

    inner_state.insert(addr1, 42);

    let state: LedgerState = inner_state.into();

    assert_eq!(state.get_or_zero(&addr2), 0);
}

#[test]
fn get_or_zero() {
    let mut inner_state = HashMap::new();
    let mut addrs = Vec::new();

    for i in 0..100 {
        let addr = rand_trits_field::<Address>();
        addrs.push(addr.clone());
        if i % 2 == 0 {
            inner_state.insert(addr, i);
        }
    }

    let state: LedgerState = inner_state.into();

    for (i, addr) in addrs.iter().enumerate() {
        if i % 2 == 0 {
            assert_eq!(state.get_or_zero(addr), i as u64);
        } else {
            assert_eq!(state.get_or_zero(addr), 0u64);
        }
    }
}

#[test]
fn apply_single_diff_mutate() {
    let mut inner_state = HashMap::new();
    let addr1 = rand_trits_field::<Address>();
    let addr2 = rand_trits_field::<Address>();

    inner_state.insert(addr1.clone(), 21);
    inner_state.insert(addr2.clone(), 63);

    let mut state: LedgerState = inner_state.into();

    state.apply_single_diff(addr1.clone(), 21);
    state.apply_single_diff(addr2.clone(), -21);

    assert_eq!(state.get_or_zero(&addr1), 42);
    assert_eq!(state.get_or_zero(&addr2), 42);
}

#[test]
fn apply_single_diff_add() {
    let inner_state = HashMap::new();
    let addr = rand_trits_field::<Address>();

    let mut state: LedgerState = inner_state.into();

    state.apply_single_diff(addr.clone(), 42);

    assert_eq!(state.get_or_zero(&addr), 42);
}

#[test]
fn apply_diff() {
    let mut inner_state = HashMap::new();
    let mut inner_diff = HashMap::new();
    let mut addrs = Vec::new();

    for i in 0..100 {
        let addr = rand_trits_field::<Address>();
        addrs.push(addr.clone());
        inner_state.insert(addr.clone(), 200);
        if i % 3 == 0 {
            inner_diff.insert(addr, i);
        } else if i % 3 == 1 {
            inner_diff.insert(addr, -i);
        }
    }

    let mut state: LedgerState = inner_state.into();
    let diff: LedgerDiff = inner_diff.into();

    state.apply_diff(diff);

    for (i, addr) in addrs.iter().enumerate() {
        if i % 3 == 0 {
            assert_eq!(state.get_or_zero(addr), (200 + i) as u64);
        } else if i % 3 == 1 {
            assert_eq!(state.get_or_zero(addr), (200 - i) as u64);
        } else {
            assert_eq!(state.get_or_zero(addr), 200 as u64);
        }
    }
}

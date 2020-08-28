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

mod helpers;

use self::helpers::*;

use bee_tangle::traversal::*;

#[test]
fn visit_children_follow_trunk_in_simple_graph() {
    // a   b0
    // |\ /
    // | c1
    // |/|
    // d2|
    //  \|
    //   e2

    let (tangle, Transactions { b, c, d, e, .. }, Hashes { b_hash, .. }) = create_test_tangle();

    let mut txs = vec![];

    visit_children_follow_trunk(&tangle, b_hash, |_, _| true, |_, tx, _| txs.push(tx.clone()));

    assert_eq!(4, txs.len());

    assert_eq!(b.address(), txs[0].address());
    assert_eq!(c.address(), txs[1].address());
    assert!(d.address() == txs[2].address() || d.address() == txs[3].address());
    assert!(e.address() == txs[2].address() || e.address() == txs[3].address());
}

#[test]
fn visit_parents_follow_trunk_in_simple_graph() {
    // a   b2
    // |\ /
    // | c1
    // |/|
    // d |
    //  \|
    //   e0

    let (tangle, Transactions { e, b, c, d, .. }, Hashes { d_hash, e_hash, .. }) = create_test_tangle();

    let mut txs = vec![];

    visit_parents_follow_trunk(&tangle, e_hash, |_, _| true, |_, tx, _| txs.push(tx.clone()));

    assert_eq!(3, txs.len());

    assert_eq!(e.address(), txs[0].address());
    assert_eq!(c.address(), txs[1].address());
    assert_eq!(b.address(), txs[2].address());

    txs.clear();

    // a   b2
    // |\ /
    // | c1
    // |/|
    // d0|
    //  \|
    //   e
    visit_parents_follow_trunk(&tangle, d_hash, |_, _| true, |_, tx, _| txs.push(tx.clone()));

    assert_eq!(d.address(), txs[0].address());
    assert_eq!(c.address(), txs[1].address());
    assert_eq!(b.address(), txs[2].address());
}

#[test]
fn visit_parents_depth_first_in_simple_graph() {
    // a2  b4
    // |\ /
    // | c3
    // |/|
    // d1|
    //  \|
    //   e0

    let (tangle, Transactions { a, b, c, d, e, .. }, Hashes { e_hash, .. }) = create_test_tangle();

    let mut addresses = vec![];

    visit_parents_depth_first(
        &tangle,
        e_hash,
        |_, _, _| true,
        |_, data, _| addresses.push(data.address().clone()),
        |_| (),
    );

    assert_eq!(5, addresses.len());

    assert_eq!(*e.address(), addresses[0]);
    assert_eq!(*d.address(), addresses[1]);
    assert_eq!(*a.address(), addresses[2]);
    assert_eq!(*c.address(), addresses[3]);
    assert_eq!(*b.address(), addresses[4]);
}

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

use crate::transaction::{create_random_attached_tx, create_random_tx};

use bee_tangle::Tangle;
use bee_transaction::{BundledTransaction as Tx, Hash as TxHash, TransactionVertex};

pub struct Transactions {
    pub a: Tx,
    pub b: Tx,
    pub c: Tx,
    pub d: Tx,
    pub e: Tx,
}

pub struct Hashes {
    pub a_hash: TxHash,
    pub b_hash: TxHash,
    pub c_hash: TxHash,
    pub d_hash: TxHash,
    pub e_hash: TxHash,
}

#[allow(clippy::many_single_char_names)]
pub fn create_test_tangle() -> (Tangle<()>, Transactions, Hashes) {
    // a   b
    // |\ /
    // | c
    // |/|
    // d |
    //  \|
    //   e

    let tangle = Tangle::new();

    let (a_hash, a) = create_random_tx();
    let (b_hash, b) = create_random_tx();
    let (c_hash, c) = create_random_attached_tx(a_hash.clone(), b_hash.clone());
    let (d_hash, d) = create_random_attached_tx(a_hash.clone(), c_hash.clone());
    let (e_hash, e) = create_random_attached_tx(d_hash.clone(), c_hash.clone());

    assert_eq!(*c.trunk(), b_hash);
    assert_eq!(*c.branch(), a_hash);
    assert_eq!(*d.trunk(), c_hash);
    assert_eq!(*d.branch(), a_hash);
    assert_eq!(*e.trunk(), c_hash);
    assert_eq!(*e.branch(), d_hash);

    tangle.insert(a.clone(), a_hash, ());
    tangle.insert(b.clone(), b_hash, ());
    tangle.insert(c.clone(), c_hash, ());
    tangle.insert(d.clone(), d_hash, ());
    tangle.insert(e.clone(), e_hash, ());

    assert_eq!(*tangle.get(&c_hash).unwrap().trunk(), b_hash);
    assert_eq!(*tangle.get(&c_hash).unwrap().branch(), a_hash);
    assert_eq!(*tangle.get(&d_hash).unwrap().trunk(), c_hash);
    assert_eq!(*tangle.get(&d_hash).unwrap().branch(), a_hash);
    assert_eq!(*tangle.get(&e_hash).unwrap().trunk(), c_hash);
    assert_eq!(*tangle.get(&e_hash).unwrap().branch(), d_hash);

    // TODO ensure children reference their parents correctly

    assert_eq!(5, tangle.size());
    assert_eq!(2, tangle.num_children(&a_hash));
    assert_eq!(1, tangle.num_children(&b_hash));
    assert_eq!(2, tangle.num_children(&c_hash));
    assert_eq!(1, tangle.num_children(&d_hash));
    assert_eq!(0, tangle.num_children(&e_hash));

    (
        tangle,
        Transactions { a, b, c, d, e },
        Hashes {
            a_hash,
            b_hash,
            c_hash,
            d_hash,
            e_hash,
        },
    )
}

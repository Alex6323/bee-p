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

use crate::{tangle::Tangle, vertex::Vertex};

use bee_transaction::Hash as TxHash;

use std::collections::HashSet;

// TODO: rename function name -> walk_past
// TODO: rename Filter -> Match
// TODO: rename Action -> Apply
pub fn trunk_walk_approvees<'a, T, Filter, Action>(tangle: &'a Tangle<T>, start: TxHash, f: Filter, mut a: Action)
where
    Filter: Fn(&Vertex<T>) -> bool,
    Action: FnMut(&TxHash, &Vertex<T>),
    T: Clone + Copy,
{
    let mut approvers = vec![start];

    // TODO: optimize
    while let Some(approver_hash) = approvers.pop() {
        if let Some(r) = tangle.vertices.get(&approver_hash) {
            let vtx = r.value();

            if !f(vtx) {
                break;
            } else {
                a(&approver_hash, vtx);
                approvers.push(vtx.get_trunk().clone());
            }
        }
    }
}

// TODO: rename 'walk_approvees_depth_first'
pub fn df_walk_approvees<'a, T, Follow, Hit, Miss>(
    tangle: &'a Tangle<T>,
    start: TxHash,
    f: Follow,
    mut h: Hit,
    mut m: Miss,
) where
    Follow: Fn(&TxHash, &Vertex<T>) -> bool,
    Hit: FnMut(&TxHash, &Vertex<T>),
    Miss: FnMut(&TxHash),
    T: Clone + Copy,
{
    let mut to_visit = Vec::new();
    let mut visited = HashSet::new();

    to_visit.push(start);

    while let Some(hash) = to_visit.pop() {
        if !visited.contains(&hash) {
            match tangle.vertices.get(&hash) {
                Some(vtx) => {
                    let vtx = vtx.value();

                    h(&hash, vtx);

                    if f(&hash, vtx) {
                        to_visit.push(*vtx.get_trunk());
                        to_visit.push(*vtx.get_branch());
                    }
                }
                None => {
                    // TODO: need to handle this in protocol
                    // if !self.is_solid_entry_point(&hash) {
                    m(&hash);
                    //}
                }
            }
            visited.insert(hash);
        }
    }
}

// TODO: rename Filter -> Continue
// TODO: rename function name -> walk_approvers_trunk_breadth_first
pub fn trunk_walk_approvers<'a, T, Filter, Action>(tangle: &'a Tangle<T>, start: TxHash, f: Filter, mut a: Action)
where
    Filter: Fn(&Vertex<T>) -> bool,
    Action: FnMut(&TxHash, &Vertex<T>),
    T: Clone + Copy,
{
    let mut to_visit = vec![];

    // NOTE: do we need to do this for `start`?
    tangle.vertices.get(&start).map(|r| {
        if f(r.value()) {
            to_visit.push(start);
            a(&start, r.value());
        }
    });

    while let Some(hash) = to_visit.pop() {
        if let Some(r) = tangle.approvers.get(&hash) {
            for approver_hash in r.value() {
                if let Some(s) = tangle.vertices.get(approver_hash) {
                    if s.get_trunk() == &hash && f(s.value()) {
                        to_visit.push(*approver_hash);
                        a(approver_hash, s.value());
                        // NOTE: For simplicity reasons we break here, and assume, that there can't be
                        // a second approver that passes the filter
                        break;
                    }
                }
            }
        }
    }
}

pub fn df_walk_approvers<'a, T, Filter, Hit, Miss>(
    tangle: &'a Tangle<T>,
    start: TxHash,
    f: Filter,
    mut h: Hit,
    mut m: Miss,
) where
    Hit: FnMut(&TxHash, &Vertex<T>),
    Filter: Fn(&TxHash, &Vertex<T>) -> bool,
    Miss: FnMut(&TxHash),
    T: Clone + Copy,
{
    let mut to_visit = vec![start];
    let mut visited = HashSet::new();

    while let Some(hash) = to_visit.last() {
        match tangle.vertices.get(hash) {
            Some(r) => {
                let vtx = r.value();

                if visited.contains(vtx.get_trunk()) && visited.contains(vtx.get_branch()) {
                    h(hash, &vtx);
                    visited.insert(hash.clone());
                    to_visit.pop();
                } else if !visited.contains(vtx.get_trunk()) {
                    if f(hash, &vtx) {
                        to_visit.push(vtx.get_trunk().clone());
                    }
                } else if !visited.contains(vtx.get_branch()) {
                    if f(hash, &vtx) {
                        to_visit.push(vtx.get_branch().clone());
                    }
                }
            }
            None => {
                // NOTE: this has to be dealt at the protocol level now ;)
                // if !tangle.solid_entry_points.contains(hash) {
                m(hash);
                //}
                visited.insert(hash.clone());
                to_visit.pop();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::tangle::Tangle;

    use bee_transaction::BundledTransaction as Tx;

    use bee_test::transaction::{create_random_attached_tx, create_random_tx};

    #[test]
    fn walk_trunk_approvers() {
        let (tangle, Transactions { a, d, e, .. }, Hashes { a_hash, .. }) = create_test_tangle();

        let mut txs = vec![];

        trunk_walk_approvers(
            &tangle,
            a_hash,
            |_vtx| true,
            |_hash, vtx| txs.push(vtx.get_transaction().clone()),
        );

        assert_eq!(3, txs.len());

        assert_eq!(a.address(), txs[0].address());
        assert_eq!(d.address(), txs[1].address());
        assert_eq!(e.address(), txs[2].address());
    }

    #[test]
    fn walk_trunk_approvees() {
        let (tangle, Transactions { a, d, e, .. }, Hashes { e_hash, .. }) = create_test_tangle();

        let mut txs = vec![];

        trunk_walk_approvees(
            &tangle,
            e_hash,
            |_vtx| true,
            |_hash, vtx| txs.push(vtx.get_transaction().clone()),
        );

        assert_eq!(3, txs.len());

        assert_eq!(e.address(), txs[0].address());
        assert_eq!(d.address(), txs[1].address());
        assert_eq!(a.address(), txs[2].address());
    }

    #[test]
    fn test_df_walk_approvees() {
        let (tangle, Transactions { a, b, c, d, e, .. }, Hashes { e_hash, .. }) = create_test_tangle();

        let mut addresses = vec![];

        df_walk_approvees(
            &tangle,
            e_hash,
            |_hash, _vtx| true,
            |_hash, vtx| addresses.push(vtx.get_transaction().address().clone()),
            |_hash| (),
        );

        assert_eq!(*e.address(), addresses[0]);
        assert_eq!(*d.address(), addresses[1]);
        assert_eq!(*a.address(), addresses[2]);
        assert_eq!(*c.address(), addresses[3]);
        assert_eq!(*b.address(), addresses[4]);
    }

    struct Transactions {
        pub a: Tx,
        pub b: Tx,
        pub c: Tx,
        pub d: Tx,
        pub e: Tx,
    }

    struct Hashes {
        pub a_hash: TxHash,
        pub b_hash: TxHash,
        pub c_hash: TxHash,
        pub d_hash: TxHash,
        pub e_hash: TxHash,
    }

    #[allow(clippy::many_single_char_names)]
    fn create_test_tangle() -> (Tangle<()>, Transactions, Hashes) {
        // a   b
        // |\ /
        // | c
        // |/|
        // d |
        //  \|
        //   e
        //
        // Trunk path from 'e':
        // e --(trunk)-> d --(trunk)-> a

        let tangle = Tangle::new();

        let (a_hash, a) = create_random_tx();
        let (b_hash, b) = create_random_tx();
        let (c_hash, c) = create_random_attached_tx(a_hash.clone(), b_hash.clone()); // branch, trunk
        let (d_hash, d) = create_random_attached_tx(c_hash.clone(), a_hash.clone());
        let (e_hash, e) = create_random_attached_tx(c_hash.clone(), d_hash.clone());

        tangle.insert_transaction(a.clone(), a_hash, ());
        tangle.insert_transaction(b.clone(), b_hash, ());
        tangle.insert_transaction(c.clone(), c_hash, ());
        tangle.insert_transaction(d.clone(), d_hash, ());
        tangle.insert_transaction(e.clone(), e_hash, ());

        assert_eq!(5, tangle.size());
        assert_eq!(2, tangle.num_approvers(&a_hash));
        assert_eq!(1, tangle.num_approvers(&b_hash));
        assert_eq!(2, tangle.num_approvers(&c_hash));
        assert_eq!(1, tangle.num_approvers(&d_hash));
        assert_eq!(0, tangle.num_approvers(&e_hash));

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

    // #[test]
    // #[serial]
    // fn walk_approvers_post_order_dfs() {
    // Example from https://github.com/iotaledger/protocol-rfcs/blob/master/text/0005-white-flag/0005-white-flag.md
    //
    // init();
    //
    // let tangle = tangle();
    //
    // Creates solid entry points
    // let sep1 = rand_trits_field::<Hash>();
    // let sep2 = rand_trits_field::<Hash>();
    // let sep3 = rand_trits_field::<Hash>();
    // let sep4 = rand_trits_field::<Hash>();
    // let sep5 = rand_trits_field::<Hash>();
    // let sep6 = rand_trits_field::<Hash>();
    // tangle.add_solid_entry_point(sep1);
    // tangle.add_solid_entry_point(sep2);
    // tangle.add_solid_entry_point(sep3);
    // tangle.add_solid_entry_point(sep4);
    // tangle.add_solid_entry_point(sep5);
    // tangle.add_solid_entry_point(sep6);
    //
    // Links transactions
    // let (a_hash, a) = create_random_attached_tx(sep1, sep2);
    // let (b_hash, b) = create_random_attached_tx(sep3, sep4);
    // let (c_hash, c) = create_random_attached_tx(sep5, sep6);
    // let (d_hash, d) = create_random_attached_tx(b_hash, a_hash);
    // let (e_hash, e) = create_random_attached_tx(b_hash, a_hash);
    // let (f_hash, f) = create_random_attached_tx(c_hash, b_hash);
    // let (g_hash, g) = create_random_attached_tx(e_hash, d_hash);
    // let (h_hash, h) = create_random_attached_tx(f_hash, e_hash);
    // let (i_hash, i) = create_random_attached_tx(c_hash, f_hash);
    // let (j_hash, j) = create_random_attached_tx(h_hash, g_hash);
    // let (k_hash, k) = create_random_attached_tx(i_hash, h_hash);
    // let (l_hash, l) = create_random_attached_tx(j_hash, g_hash);
    // let (m_hash, m) = create_random_attached_tx(h_hash, j_hash);
    // let (n_hash, n) = create_random_attached_tx(k_hash, h_hash);
    // let (o_hash, o) = create_random_attached_tx(i_hash, k_hash);
    // let (p_hash, p) = create_random_attached_tx(i_hash, k_hash);
    // let (q_hash, q) = create_random_attached_tx(m_hash, l_hash);
    // let (r_hash, r) = create_random_attached_tx(m_hash, l_hash);
    // let (s_hash, s) = create_random_attached_tx(o_hash, n_hash);
    // let (t_hash, t) = create_random_attached_tx(p_hash, o_hash);
    // let (u_hash, u) = create_random_attached_tx(r_hash, q_hash);
    // let (v_hash, v) = create_random_attached_tx(s_hash, r_hash);
    // let (w_hash, w) = create_random_attached_tx(t_hash, s_hash);
    // let (x_hash, x) = create_random_attached_tx(u_hash, q_hash);
    // let (y_hash, y) = create_random_attached_tx(v_hash, u_hash);
    // let (z_hash, z) = create_random_attached_tx(s_hash, v_hash);
    //
    // Confirms transactions
    // TODO uncomment when confirmation index
    // tangle.confirm_transaction(a_hash, 1);
    // tangle.confirm_transaction(b_hash, 1);
    // tangle.confirm_transaction(c_hash, 1);
    // tangle.confirm_transaction(d_hash, 2);
    // tangle.confirm_transaction(e_hash, 1);
    // tangle.confirm_transaction(f_hash, 1);
    // tangle.confirm_transaction(g_hash, 2);
    // tangle.confirm_transaction(h_hash, 1);
    // tangle.confirm_transaction(i_hash, 2);
    // tangle.confirm_transaction(j_hash, 2);
    // tangle.confirm_transaction(k_hash, 2);
    // tangle.confirm_transaction(l_hash, 2);
    // tangle.confirm_transaction(m_hash, 2);
    // tangle.confirm_transaction(n_hash, 2);
    // tangle.confirm_transaction(o_hash, 2);
    // tangle.confirm_transaction(p_hash, 3);
    // tangle.confirm_transaction(q_hash, 3);
    // tangle.confirm_transaction(r_hash, 2);
    // tangle.confirm_transaction(s_hash, 2);
    // tangle.confirm_transaction(t_hash, 3);
    // tangle.confirm_transaction(u_hash, 3);
    // tangle.confirm_transaction(v_hash, 2);
    // tangle.confirm_transaction(w_hash, 3);
    // tangle.confirm_transaction(x_hash, 3);
    // tangle.confirm_transaction(y_hash, 3);
    // tangle.confirm_transaction(z_hash, 3);
    //
    // Constructs the graph
    // block_on(async {
    // tangle.insert_transaction(a, a_hash).await;
    // tangle.insert_transaction(b, b_hash).await;
    // tangle.insert_transaction(c, c_hash).await;
    // tangle.insert_transaction(d, d_hash).await;
    // tangle.insert_transaction(e, e_hash).await;
    // tangle.insert_transaction(f, f_hash).await;
    // tangle.insert_transaction(g, g_hash).await;
    // tangle.insert_transaction(h, h_hash).await;
    // tangle.insert_transaction(i, i_hash).await;
    // tangle.insert_transaction(j, j_hash).await;
    // tangle.insert_transaction(k, k_hash).await;
    // tangle.insert_transaction(l, l_hash).await;
    // tangle.insert_transaction(m, m_hash).await;
    // tangle.insert_transaction(n, n_hash).await;
    // tangle.insert_transaction(o, o_hash).await;
    // tangle.insert_transaction(p, p_hash).await;
    // tangle.insert_transaction(q, q_hash).await;
    // tangle.insert_transaction(r, r_hash).await;
    // tangle.insert_transaction(s, s_hash).await;
    // tangle.insert_transaction(t, t_hash).await;
    // tangle.insert_transaction(u, u_hash).await;
    // tangle.insert_transaction(v, v_hash).await;
    // tangle.insert_transaction(w, w_hash).await;
    // tangle.insert_transaction(x, x_hash).await;
    // tangle.insert_transaction(y, y_hash).await;
    // tangle.insert_transaction(z, z_hash).await;
    // });
    //
    // let mut hashes = Vec::new();
    //
    // tangle.walk_approvers_post_order_dfs(
    // v_hash,
    // |hash, _transaction| {
    // hashes.push(*hash);
    // ()
    // },
    // |_| true,
    // |_| (),
    // );
    //
    // TODO Remove when we have confirmation index
    // assert_eq!(hashes.len(), 18);
    // assert_eq!(hashes[0], a_hash);
    // assert_eq!(hashes[1], b_hash);
    // assert_eq!(hashes[2], d_hash);
    // assert_eq!(hashes[3], e_hash);
    // assert_eq!(hashes[4], g_hash);
    // assert_eq!(hashes[5], c_hash);
    // assert_eq!(hashes[6], f_hash);
    // assert_eq!(hashes[7], h_hash);
    // assert_eq!(hashes[8], j_hash);
    // assert_eq!(hashes[9], l_hash);
    // assert_eq!(hashes[10], m_hash);
    // assert_eq!(hashes[11], r_hash);
    // assert_eq!(hashes[12], i_hash);
    // assert_eq!(hashes[13], k_hash);
    // assert_eq!(hashes[14], n_hash);
    // assert_eq!(hashes[15], o_hash);
    // assert_eq!(hashes[16], s_hash);
    // assert_eq!(hashes[17], v_hash);
    //
    // TODO uncomment when we have confirmation index
    // assert_eq!(hashes.len(), 12);
    // assert_eq!(hashes[0], d_hash);
    // assert_eq!(hashes[1], g_hash);
    // assert_eq!(hashes[2], j_hash);
    // assert_eq!(hashes[3], l_hash);
    // assert_eq!(hashes[4], m_hash);
    // assert_eq!(hashes[5], r_hash);
    // assert_eq!(hashes[6], i_hash);
    // assert_eq!(hashes[7], k_hash);
    // assert_eq!(hashes[8], n_hash);
    // assert_eq!(hashes[9], o_hash);
    // assert_eq!(hashes[10], s_hash);
    // assert_eq!(hashes[11], v_hash);
    //
    // drop();
    // }
}

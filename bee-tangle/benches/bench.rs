#[macro_use]
extern crate criterion;

use criterion::Criterion;

use bee_tangle::Tangle;

fn bench_insert_transaction(c: &mut Criterion) {
    todo!("insert a few thousand transactions as fast as possible")
}

criterion_group!(benches, bench_insert_transaction);
criterion_main!(benches);

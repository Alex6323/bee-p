use crate::transaction::Transaction;

/// A newtype to represent a number of transactions, that hides the internal data layout.
pub struct Transactions(Vec<Transaction>);

/// `Bundle`s are messages on the network of one or more `Transactions`s, which in turn are setnt one at a time and are stored in a distributed ledger called the `Tangle`.
///
/// For a `Bundle` to be bulidable, all required transactions have to be present when validating and building. Otherwise the build will fail.
pub struct Bundle {
    transactions: Transactions,
}

impl Transactions {
    pub fn push(&mut self, transaction: Transaction) {
        self.0.push(transaction);
    }
}

impl Bundle {
    pub fn transactions(&self) -> &Transactions {
        &self.transactions
    }
}

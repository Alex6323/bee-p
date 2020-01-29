use crate::transaction::{Transaction, TransactionBuilder};

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

/// A newtype to represent a number of transactions, that hides the internal data layout.
pub struct TransactionBuilders(Vec<TransactionBuilder>);

/// Concerned with constructing and verifying complete messages coming in externally.
struct IncomingBundleBuilder {
    transaction_builders: TransactionBuilders,
}

impl IncomingBundleBuilder {
    /// Pushes a new transaction coming over the wire into the bundle builder.
    pub fn push(&mut self, transaction_builder: TransactionBuilder) -> &mut Self {
        self.transaction_builders.push(transaction_builder);
        self
    }
}

/// Responsible for constructing a `Bundle` from scratch to be sent to the IOTA network. This includes siging its transactions, calculating the bundle hash, and setting other releveant fields depending on context.
struct OutgoingBundleBuilder;
struct SealedBundleBuilder;
struct SignedBundleBuilder;

struct AttachedBundleBuilder;
struct ValidatedBundleBuilder;

impl TransactionBuilders {
    pub fn push(&mut self, transaction_builder: TransactionBuilder) {
        self.0.push(transaction_builder);
    }
}

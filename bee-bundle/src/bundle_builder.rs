use crate::transaction::TransactionBuilder;

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

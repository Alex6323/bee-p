use crate::{Transaction, TransactionBuilder};
use std::marker::PhantomData;

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
#[derive(Default)]
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

impl TransactionBuilders {
    pub fn push(&mut self, transaction_builder: TransactionBuilder) {
        self.0.push(transaction_builder);
    }
}

////////////////////

#[derive(Default)]
struct Raw {}
struct Sealed {}
struct Signed {}
struct Attached {}
struct Validated {}

#[derive(Default)]
struct StagedOutgoingBundleBuilder<S> {
    transaction_builders: TransactionBuilders,
    build_stage: PhantomData<S>,
}

type OutgoingBundleBuilder = StagedOutgoingBundleBuilder<Raw>;

#[derive(Debug)]
enum BundleBuilderError {}

impl StagedOutgoingBundleBuilder<Raw> {
    pub fn new() -> StagedOutgoingBundleBuilder<Raw> {
        StagedOutgoingBundleBuilder::<Raw>::default()
    }

    pub fn seal(self) -> Result<StagedOutgoingBundleBuilder<Sealed>, BundleBuilderError> {
        Ok(StagedOutgoingBundleBuilder::<Sealed> {
            transaction_builders: self.transaction_builders,
            build_stage: PhantomData,
        })
    }
}

impl StagedOutgoingBundleBuilder<Sealed> {
    pub fn sign(self) -> Result<StagedOutgoingBundleBuilder<Signed>, BundleBuilderError> {
        Ok(StagedOutgoingBundleBuilder::<Signed> {
            transaction_builders: self.transaction_builders,
            build_stage: PhantomData,
        })
    }
}

impl StagedOutgoingBundleBuilder<Signed> {
    pub fn attach(self) -> Result<StagedOutgoingBundleBuilder<Attached>, BundleBuilderError> {
        Ok(StagedOutgoingBundleBuilder::<Attached> {
            transaction_builders: self.transaction_builders,
            build_stage: PhantomData,
        })
    }
}

impl StagedOutgoingBundleBuilder<Attached> {
    pub fn validate(self) -> Result<StagedOutgoingBundleBuilder<Validated>, BundleBuilderError> {
        Ok(StagedOutgoingBundleBuilder::<Validated> {
            transaction_builders: self.transaction_builders,
            build_stage: PhantomData,
        })
    }
}

impl StagedOutgoingBundleBuilder<Validated> {
    pub fn build(self) -> Result<Bundle, BundleBuilderError> {
        Ok(Bundle {
            transactions: Transactions(vec![]),
        })
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn empty_test() -> Result<(), BundleBuilderError> {
        let bundle = OutgoingBundleBuilder::new()
            .seal()?
            .sign()?
            .attach()?
            .validate()?
            .build()?;

        Ok(())
    }
}

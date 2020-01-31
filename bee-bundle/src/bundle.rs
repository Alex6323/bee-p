use crate::{Transaction, TransactionBuilder};
use std::marker::PhantomData;

/// A newtype to represent a number of transactions, that hides the internal data layout.
#[derive(Default)]
pub struct Transactions(Vec<Transaction>);

/// `Bundle`s are messages on the network of one or more `Transactions`s, which in turn are setnt one at a time and are stored in a distributed ledger called the `Tangle`.
///
/// For a `Bundle` to be bulidable, all required transactions have to be present when validating and building. Otherwise the build will fail.
pub struct Bundle {
    transactions: Transactions,
}

impl Transactions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(&mut self, transaction: Transaction) {
        self.0.push(transaction);
    }
}

impl Bundle {
    pub fn transactions(&self) -> &Transactions {
        &self.transactions
    }

    pub fn len(&self) -> usize {
        self.transactions.0.len()
    }
}

/// A newtype to represent a number of transactions, that hides the internal data layout.
#[derive(Default)]
pub struct TransactionBuilders(Vec<TransactionBuilder>);

/// Concerned with constructing and verifying complete messages coming in externally.
struct IncomingBundleBuilder {
    builders: TransactionBuilders,
}

impl IncomingBundleBuilder {
    /// Pushes a new transaction coming over the wire into the bundle builder.
    pub fn push(&mut self, transaction_builder: TransactionBuilder) -> &mut Self {
        self.builders.push(transaction_builder);
        self
    }
}

// TODO should be in tx module ?
impl TransactionBuilders {
    pub fn push(&mut self, transaction_builder: TransactionBuilder) {
        self.0.push(transaction_builder);
    }
}

////////////////////

#[derive(Default)]
struct Raw;
struct Sealed;
struct Signed;
struct Attached;
struct Validated;

//////////////////////

#[derive(Default)]
struct StagedOutgoingBundleBuilder<S> {
    builders: TransactionBuilders,
    stage: PhantomData<S>,
}

type OutgoingBundleBuilder = StagedOutgoingBundleBuilder<Raw>;

#[derive(Debug)]
enum BundleBuilderError {}

impl StagedOutgoingBundleBuilder<Raw> {
    pub fn new() -> StagedOutgoingBundleBuilder<Raw> {
        StagedOutgoingBundleBuilder::<Raw>::default()
    }

    pub fn push(&mut self, transaction_builder: TransactionBuilder) {
        self.builders.push(transaction_builder);
    }

    pub fn seal(self) -> Result<StagedOutgoingBundleBuilder<Sealed>, BundleBuilderError> {
        Ok(StagedOutgoingBundleBuilder::<Sealed> {
            builders: self.builders,
            stage: PhantomData,
        })
    }
}

impl StagedOutgoingBundleBuilder<Sealed> {
    pub fn sign(self) -> Result<StagedOutgoingBundleBuilder<Signed>, BundleBuilderError> {
        Ok(StagedOutgoingBundleBuilder::<Signed> {
            builders: self.builders,
            stage: PhantomData,
        })
    }
}

impl StagedOutgoingBundleBuilder<Signed> {
    pub fn attach(self) -> Result<StagedOutgoingBundleBuilder<Attached>, BundleBuilderError> {
        Ok(StagedOutgoingBundleBuilder::<Attached> {
            builders: self.builders,
            stage: PhantomData,
        })
    }
}

impl StagedOutgoingBundleBuilder<Attached> {
    pub fn validate(self) -> Result<StagedOutgoingBundleBuilder<Validated>, BundleBuilderError> {
        Ok(StagedOutgoingBundleBuilder::<Validated> {
            builders: self.builders,
            stage: PhantomData,
        })
    }
}

impl StagedOutgoingBundleBuilder<Validated> {
    pub fn build(self) -> Result<Bundle, BundleBuilderError> {
        let mut transactions = Transactions::new();

        for transaction_builder in self.builders.0 {
            transactions.push(transaction_builder.build());
        }

        Ok(Bundle {
            transactions: transactions,
        })
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn empty_test() -> Result<(), BundleBuilderError> {
        let mut bundle_builder = OutgoingBundleBuilder::new();

        for _ in 0..5 {
            bundle_builder.push(TransactionBuilder::default());
        }

        let bundle = bundle_builder
            .seal()?
            .sign()?
            .attach()?
            .validate()?
            .build()?;

        assert_eq!(bundle.len(), 5);

        Ok(())
    }
}

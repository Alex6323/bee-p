use crate::{Transaction, TransactionBuilder};
use crypto::Sponge;
use std::marker::PhantomData;
use ternary::TritsBuf;

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

#[derive(Debug)]
pub enum BundleBuilderError {}

///
/// Outgoing bundles
///

#[derive(Default)]
pub struct Raw;
pub struct Sealed;
pub struct Signed;
pub struct Attached;
pub struct Validated;

#[derive(Default)]
pub struct StagedOutgoingBundleBuilder<E, H, S> {
    builders: TransactionBuilders,
    essence_sponge: PhantomData<E>,
    pow_sponge: PhantomData<H>,
    stage: PhantomData<S>,
}

pub type OutgoingBundleBuilderSponge<E, H> = StagedOutgoingBundleBuilder<E, H, Raw>;
// TODO default to Kerl
pub type OutgoingBundleBuilder = OutgoingBundleBuilderSponge<crypto::CurlP81, crypto::CurlP81>;

// TODO constraint on S ?
impl<E, H, S> StagedOutgoingBundleBuilder<E, H, S>
where
    E: Sponge + Default,
    H: Sponge + Default,
{
    pub fn calculate_hash(&self) -> TritsBuf {
        let mut sponge = E::default();

        for builder in &self.builders.0 {
            // TODO sponge.absorb(builder.essence());
        }

        sponge.squeeze()
    }
}

impl<E, H> StagedOutgoingBundleBuilder<E, H, Raw>
where
    E: Sponge + Default,
    H: Sponge + Default,
{
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(&mut self, builder: TransactionBuilder) {
        self.builders.push(builder);
    }

    pub fn seal(self) -> Result<StagedOutgoingBundleBuilder<E, H, Sealed>, BundleBuilderError> {
        Ok(StagedOutgoingBundleBuilder::<E, H, Sealed> {
            builders: self.builders,
            essence_sponge: PhantomData,
            pow_sponge: PhantomData,
            stage: PhantomData,
        })
    }
}

impl<E, H> StagedOutgoingBundleBuilder<E, H, Sealed>
where
    E: Sponge + Default,
    H: Sponge + Default,
{
    pub fn sign(self) -> Result<StagedOutgoingBundleBuilder<E, H, Signed>, BundleBuilderError> {
        Ok(StagedOutgoingBundleBuilder::<E, H, Signed> {
            builders: self.builders,
            essence_sponge: PhantomData,
            pow_sponge: PhantomData,
            stage: PhantomData,
        })
    }
}

impl<E, H> StagedOutgoingBundleBuilder<E, H, Signed>
where
    E: Sponge + Default,
    H: Sponge + Default,
{
    pub fn attach(self) -> Result<StagedOutgoingBundleBuilder<E, H, Attached>, BundleBuilderError> {
        Ok(StagedOutgoingBundleBuilder::<E, H, Attached> {
            builders: self.builders,
            essence_sponge: PhantomData,
            pow_sponge: PhantomData,
            stage: PhantomData,
        })
    }
}

impl<E, H> StagedOutgoingBundleBuilder<E, H, Attached>
where
    E: Sponge + Default,
    H: Sponge + Default,
{
    pub fn validate(
        self,
    ) -> Result<StagedOutgoingBundleBuilder<E, H, Validated>, BundleBuilderError> {
        Ok(StagedOutgoingBundleBuilder::<E, H, Validated> {
            builders: self.builders,
            essence_sponge: PhantomData,
            pow_sponge: PhantomData,
            stage: PhantomData,
        })
    }
}

impl<E, H> StagedOutgoingBundleBuilder<E, H, Validated>
where
    E: Sponge + Default,
    H: Sponge + Default,
{
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

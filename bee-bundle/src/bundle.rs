use crate::{Transaction, TransactionBuilder, TransactionBuilders, Transactions};
use crypto::Sponge;
use std::marker::PhantomData;
use ternary::TritBuf;

///  Bundles

pub struct Bundle {
    transactions: Transactions,
}

impl Bundle {
    pub fn transactions(&self) -> &Transactions {
        &self.transactions
    }

    pub fn len(&self) -> usize {
        self.transactions.len()
    }
}

/// Incoming bundles

#[derive(Debug)]
pub enum IncomingBundleBuilderError {}

pub trait IncomingBundleBuilderStage {}

#[derive(Default)]
pub struct IncomingRaw;
impl IncomingBundleBuilderStage for IncomingRaw {}

pub struct IncomingValidated;
impl IncomingBundleBuilderStage for IncomingValidated {}

#[derive(Default)]
pub struct StagedIncomingBundleBuilder<E, S> {
    transactions: Transactions,
    essence_sponge: PhantomData<E>,
    stage: PhantomData<S>,
}

pub type IncomingBundleBuilderSponge<E> = StagedIncomingBundleBuilder<E, IncomingRaw>;
// TODO default kerl
pub type IncomingBundleBuilder = IncomingBundleBuilderSponge<crypto::CurlP81>;

impl<E: Sponge + Default> StagedIncomingBundleBuilder<E, IncomingRaw> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(&mut self, transactions: Transaction) {
        self.transactions.push(transactions);
    }

    pub fn validate(
        self,
    ) -> Result<StagedIncomingBundleBuilder<E, IncomingValidated>, IncomingBundleBuilderError> {
        Ok(StagedIncomingBundleBuilder::<E, IncomingValidated> {
            transactions: self.transactions,
            essence_sponge: PhantomData,
            stage: PhantomData,
        })
    }
}

impl<E: Sponge + Default> StagedIncomingBundleBuilder<E, IncomingValidated> {
    pub fn build(self) -> Bundle {
        Bundle {
            transactions: self.transactions,
        }
    }
}

/// Outgoing bundles

#[derive(Debug)]
pub enum OutgoingBundleBuilderError {}

pub trait OutgoingBundleBuilderStage {}

#[derive(Default)]
pub struct OutgoingRaw;
impl OutgoingBundleBuilderStage for OutgoingRaw {}

pub struct OutgoingSealed;
impl OutgoingBundleBuilderStage for OutgoingSealed {}

pub struct OutgoingSigned;
impl OutgoingBundleBuilderStage for OutgoingSigned {}

pub struct OutgoingAttached;
impl OutgoingBundleBuilderStage for OutgoingAttached {}

pub struct OutgoingValidated;
impl OutgoingBundleBuilderStage for OutgoingValidated {}

#[derive(Default)]
pub struct StagedOutgoingBundleBuilder<E, H, S> {
    builders: TransactionBuilders,
    essence_sponge: PhantomData<E>,
    hash_sponge: PhantomData<H>,
    stage: PhantomData<S>,
}

pub type OutgoingBundleBuilderSponge<E, H> = StagedOutgoingBundleBuilder<E, H, OutgoingRaw>;
// TODO default to Kerl
pub type OutgoingBundleBuilder = OutgoingBundleBuilderSponge<crypto::CurlP81, crypto::CurlP81>;

impl<E, H, S> StagedOutgoingBundleBuilder<E, H, S>
where
    E: Sponge + Default,
    H: Sponge + Default,
    S: OutgoingBundleBuilderStage,
{
    pub fn calculate_hash(&self) -> TritBuf {
        let mut sponge = E::default();

        for builder in &self.builders.0 {
            // TODO sponge.absorb(builder.essence());
        }

        sponge.squeeze()
    }
}

impl<E, H> StagedOutgoingBundleBuilder<E, H, OutgoingRaw>
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

    pub fn seal(
        self,
    ) -> Result<StagedOutgoingBundleBuilder<E, H, OutgoingSealed>, OutgoingBundleBuilderError> {
        // TODO Impl
        Ok(StagedOutgoingBundleBuilder::<E, H, OutgoingSealed> {
            builders: self.builders,
            essence_sponge: PhantomData,
            hash_sponge: PhantomData,
            stage: PhantomData,
        })
    }
}

impl<E, H> StagedOutgoingBundleBuilder<E, H, OutgoingSealed>
where
    E: Sponge + Default,
    H: Sponge + Default,
{
    pub fn sign(
        self,
    ) -> Result<StagedOutgoingBundleBuilder<E, H, OutgoingSigned>, OutgoingBundleBuilderError> {
        // TODO Impl
        Ok(StagedOutgoingBundleBuilder::<E, H, OutgoingSigned> {
            builders: self.builders,
            essence_sponge: PhantomData,
            hash_sponge: PhantomData,
            stage: PhantomData,
        })
    }
}

impl<E, H> StagedOutgoingBundleBuilder<E, H, OutgoingSigned>
where
    E: Sponge + Default,
    H: Sponge + Default,
{
    pub fn attach(
        self,
    ) -> Result<StagedOutgoingBundleBuilder<E, H, OutgoingAttached>, OutgoingBundleBuilderError>
    {
        // TODO Impl
        Ok(StagedOutgoingBundleBuilder::<E, H, OutgoingAttached> {
            builders: self.builders,
            essence_sponge: PhantomData,
            hash_sponge: PhantomData,
            stage: PhantomData,
        })
    }
}

impl<E, H> StagedOutgoingBundleBuilder<E, H, OutgoingAttached>
where
    E: Sponge + Default,
    H: Sponge + Default,
{
    pub fn validate(
        self,
    ) -> Result<StagedOutgoingBundleBuilder<E, H, OutgoingValidated>, OutgoingBundleBuilderError>
    {
        // TODO Impl
        Ok(StagedOutgoingBundleBuilder::<E, H, OutgoingValidated> {
            builders: self.builders,
            essence_sponge: PhantomData,
            hash_sponge: PhantomData,
            stage: PhantomData,
        })
    }
}

impl<E, H> StagedOutgoingBundleBuilder<E, H, OutgoingValidated>
where
    E: Sponge + Default,
    H: Sponge + Default,
{
    pub fn build(self) -> Result<Bundle, OutgoingBundleBuilderError> {
        // TODO Impl
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
    fn incoming_bundle_builder_test() -> Result<(), IncomingBundleBuilderError> {
        let mut bundle_builder = IncomingBundleBuilder::new();

        for _ in 0..5 {
            bundle_builder.push(Transaction::default());
        }

        let bundle = bundle_builder.validate()?.build();

        assert_eq!(bundle.len(), 5);

        Ok(())
    }

    #[test]
    fn outgoing_bundle_builder_test() -> Result<(), OutgoingBundleBuilderError> {
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

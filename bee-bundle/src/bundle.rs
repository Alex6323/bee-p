use crate::transaction::{Transaction, TransactionBuilder, TransactionBuilders, Transactions};

use std::marker::PhantomData;
use std::ops::Index;

use crypto::Sponge;
use ternary::TritsBuf;

/// Bundle

pub struct Bundle(Transactions);

impl Bundle {
    // TODO TEST
    pub fn get(&self, index: usize) -> Option<&Transaction> {
        self.0.get(index)
    }

    // TODO TEST
    pub fn len(&self) -> usize {
        self.0.len()
    }
}

impl IntoIterator for Bundle {
    type Item = Transaction;
    type IntoIter = std::vec::IntoIter<Transaction>;

    // TODO TEST
    fn into_iter(self) -> Self::IntoIter {
        (self.0).0.into_iter()
    }
}

impl<'a> IntoIterator for &'a Bundle {
    type Item = &'a Transaction;
    type IntoIter = std::slice::Iter<'a, Transaction>;

    // TODO TEST
    fn into_iter(self) -> Self::IntoIter {
        (&(self.0).0).into_iter()
    }
}

impl Index<usize> for Bundle {
    type Output = Transaction;

    // TODO TEST
    fn index(&self, index: usize) -> &Self::Output {
        self.get(index).unwrap()
    }
}

/// Incoming bundle builder

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
    // TODO TEST
    pub fn new() -> Self {
        Self::default()
    }

    // TODO TEST
    pub fn push(&mut self, transactions: Transaction) {
        self.transactions.push(transactions);
    }

    // TODO TEST
    pub fn calculate_hash(&self) -> TritsBuf {
        // TODO Impl
        let mut sponge = E::default();

        for builder in &self.transactions.0 {
            // TODO sponge.absorb(builder.essence());
        }

        sponge.squeeze()
    }

    // TODO TEST
    pub fn validate(
        self,
    ) -> Result<StagedIncomingBundleBuilder<E, IncomingValidated>, IncomingBundleBuilderError> {
        // TODO Impl
        Ok(StagedIncomingBundleBuilder::<E, IncomingValidated> {
            transactions: self.transactions,
            essence_sponge: PhantomData,
            stage: PhantomData,
        })
    }
}

impl<E: Sponge + Default> StagedIncomingBundleBuilder<E, IncomingValidated> {
    // TODO TEST
    pub fn build(self) -> Bundle {
        // TODO Impl
        Bundle(self.transactions)
    }
}

/// Outgoing bundle builder

#[derive(Debug)]
pub enum OutgoingBundleBuilderError {
    IncompleteTransactionBuilder(String),
    Empty,
    UnsignedInput,
    NonZeroSum(i64),
}

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
    // TODO TEST
    pub fn calculate_hash(&self) -> TritsBuf {
        // TODO Impl
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
    // TODO TEST
    pub fn new() -> Self {
        Self::default()
    }

    // TODO TEST
    pub fn push(&mut self, builder: TransactionBuilder) {
        self.builders.push(builder);
    }

    // TODO TEST
    pub fn seal(
        self,
    ) -> Result<StagedOutgoingBundleBuilder<E, H, OutgoingSealed>, OutgoingBundleBuilderError> {
        // TODO Impl

        for builder in &self.builders.0 {
            if let None = builder.payload {
                return Err(OutgoingBundleBuilderError::IncompleteTransactionBuilder(
                    "payload".to_owned(),
                ));
            } else if let None = builder.address {
                return Err(OutgoingBundleBuilderError::IncompleteTransactionBuilder(
                    "address".to_owned(),
                ));
            } else if let None = builder.value {
                return Err(OutgoingBundleBuilderError::IncompleteTransactionBuilder(
                    "value".to_owned(),
                ));
            } else if let None = builder.tag {
                return Err(OutgoingBundleBuilderError::IncompleteTransactionBuilder(
                    "tag".to_owned(),
                ));
            }
        }

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
    // TODO TEST
    pub fn attach(
        self,
    ) -> Result<StagedOutgoingBundleBuilder<E, H, OutgoingAttached>, OutgoingBundleBuilderError>
    {
        // TODO Impl

        for builder in &self.builders.0 {
            // Safe to unwrap because we made sure it's not None in `seal`
            if builder.value.as_ref().unwrap().0 < 0 {
                return Err(OutgoingBundleBuilderError::UnsignedInput);
            }
        }

        StagedOutgoingBundleBuilder::<E, H, OutgoingSigned> {
            builders: self.builders,
            essence_sponge: PhantomData,
            hash_sponge: PhantomData,
            stage: PhantomData,
        }
        .attach()
    }

    // TODO TEST
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
    // TODO TEST
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
    // TODO TEST
    pub fn validate(
        self,
    ) -> Result<StagedOutgoingBundleBuilder<E, H, OutgoingValidated>, OutgoingBundleBuilderError>
    {
        // TODO should call validate() on transaction builders ?
        // TODO Impl
        let mut sum: i64 = 0;

        if self.builders.len() == 0 {
            return Err(OutgoingBundleBuilderError::Empty);
        }

        // TODO unwrap ?
        for builder in &self.builders.0 {
            sum += builder.value.as_ref().unwrap().0;
        }

        if sum != 0 {
            return Err(OutgoingBundleBuilderError::NonZeroSum(sum));
        }

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
    // TODO TEST
    pub fn build(self) -> Result<Bundle, OutgoingBundleBuilderError> {
        // TODO Impl
        let mut transactions = Transactions::new();

        for transaction_builder in self.builders.0 {
            // TODO: we probably should use build()? here, and propagate possible errors
            transactions.push(transaction_builder.build_or_default());
        }

        Ok(Bundle(transactions))
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn incoming_bundle_builder_test() -> Result<(), IncomingBundleBuilderError> {
        let mut bundle_builder = IncomingBundleBuilder::new();

        for _ in 0..5 {
            bundle_builder.push(TransactionBuilder::new().build_or_default());
        }

        let bundle = bundle_builder.validate()?.build();

        assert_eq!(bundle.len(), 5);

        Ok(())
    }

    #[test]
    fn outgoing_bundle_builder_value_test() -> Result<(), OutgoingBundleBuilderError> {
        let mut bundle_builder = OutgoingBundleBuilder::new();

        for _ in 0..5 {
            bundle_builder.push(TransactionBuilder::new());
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

    #[test]
    fn outgoing_bundle_builder_data_test() -> Result<(), OutgoingBundleBuilderError> {
        let mut bundle_builder = OutgoingBundleBuilder::new();

        for _ in 0..5 {
            bundle_builder.push(TransactionBuilder::new());
        }

        let bundle = bundle_builder.seal()?.attach()?.validate()?.build()?;

        assert_eq!(bundle.len(), 5);

        Ok(())
    }
}

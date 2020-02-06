use crate::transaction::{
    Hash, Index, Transaction, TransactionBuilder, TransactionBuilders, Transactions,
};

use std::marker::PhantomData;

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

impl std::ops::Index<usize> for Bundle {
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
    IncompleteTransactionBuilder(&'static str),
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
    fn calculate_hash(&self) -> TritsBuf {
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
        mut self,
    ) -> Result<StagedOutgoingBundleBuilder<E, H, OutgoingSealed>, OutgoingBundleBuilderError> {
        // TODO Impl
        let mut index = 0;
        let last_index = self.builders.len() - 1;

        for builder in &mut self.builders.0 {
            if builder.payload.is_none() {
                return Err(OutgoingBundleBuilderError::IncompleteTransactionBuilder(
                    "payload",
                ));
            } else if builder.address.is_none() {
                return Err(OutgoingBundleBuilderError::IncompleteTransactionBuilder(
                    "address",
                ));
            } else if builder.value.is_none() {
                return Err(OutgoingBundleBuilderError::IncompleteTransactionBuilder(
                    "value",
                ));
            } else if builder.tag.is_none() {
                return Err(OutgoingBundleBuilderError::IncompleteTransactionBuilder(
                    "tag",
                ));
            }

            builder.index.replace(Index(index));
            builder.last_index.replace(Index(last_index));

            index = index + 1;
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
        trunk: Hash,
        branch: Hash,
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
        .attach(trunk, branch)
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
        trunk: Hash,
        branch: Hash,
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
    use crate::transaction::{Address, Payload, Tag, Value};

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

    // TODO Also check to attach if value ?
    #[test]
    fn outgoing_bundle_builder_value_test() -> Result<(), OutgoingBundleBuilderError> {
        let mut bundle_builder = OutgoingBundleBuilder::new();

        for _ in 0..3 {
            let transaction_builder = TransactionBuilder::new()
                .with_payload(Payload::zeros())
                .with_address(Address::zeros())
                .with_value(Value(0))
                .with_tag(Tag::zeros());
            bundle_builder.push(transaction_builder);
        }

        let bundle = bundle_builder
            .seal()?
            .sign()?
            .attach(Hash::zeros(), Hash::zeros())?
            .validate()?
            .build()?;

        assert_eq!(bundle.len(), 3);

        Ok(())
    }

    // TODO Also check to sign if data ?
    #[test]
    fn outgoing_bundle_builder_data_test() -> Result<(), OutgoingBundleBuilderError> {
        let mut bundle_builder = OutgoingBundleBuilder::new();

        for _ in 0..3 {
            let transaction_builder = TransactionBuilder::new()
                .with_payload(Payload::zeros())
                .with_address(Address::zeros())
                .with_value(Value(0))
                .with_tag(Tag::zeros());
            bundle_builder.push(transaction_builder);
        }

        let bundle = bundle_builder
            .seal()?
            .attach(Hash::zeros(), Hash::zeros())?
            .validate()?
            .build()?;

        assert_eq!(bundle.len(), 3);

        Ok(())
    }
}

use crate::transaction::{
    Hash, Index, Transaction, TransactionBuilder, TransactionBuilderError, TransactionBuilders,
    Transactions,
};

use bee_crypto::Sponge;
use bee_ternary::TritsBuf;

use std::marker::PhantomData;

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

    // TODO TEST
    pub fn hash(&self) -> &Hash {
        // Safe to unwrap because empty bundles can't be built
        self.get(0).unwrap().bundle()
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
        // Unwrap because index is expected to panic if out of range
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
pub type IncomingBundleBuilder = IncomingBundleBuilderSponge<bee_crypto::CurlP81>;

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
    // TODO common with outgoing bundle builder
    fn calculate_hash(&self) -> TritsBuf {
        // TODO Impl
        let mut sponge = E::default();

        for builder in &self.transactions.0 {
            // sponge.absorb(builder.address.0);
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
        Bundle(self.transactions)
    }
}

/// Outgoing bundle builder

#[derive(Debug)]
pub enum OutgoingBundleBuilderError {
    Empty,
    UnsignedInput,
    Inconsistent(i64),
    MissingTransactionBuilderField(&'static str),
    FailedTransactionBuild(TransactionBuilderError),
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

#[derive(Default)]
pub struct StagedOutgoingBundleBuilder<E, S> {
    builders: TransactionBuilders,
    essence_sponge: PhantomData<E>,
    stage: PhantomData<S>,
}

pub type OutgoingBundleBuilderSponge<E> = StagedOutgoingBundleBuilder<E, OutgoingRaw>;
// TODO default to Kerl
pub type OutgoingBundleBuilder = OutgoingBundleBuilderSponge<bee_crypto::CurlP81>;

impl<E, S> StagedOutgoingBundleBuilder<E, S>
where
    E: Sponge + Default,
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

impl<E: Sponge + Default> StagedOutgoingBundleBuilder<E, OutgoingRaw> {
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
    ) -> Result<StagedOutgoingBundleBuilder<E, OutgoingSealed>, OutgoingBundleBuilderError> {
        // TODO Impl
        // TODO should call validate() on transaction builders ?
        let mut sum: i64 = 0;
        let last_index = self.builders.len() - 1;

        if self.builders.len() == 0 {
            return Err(OutgoingBundleBuilderError::Empty);
        }

        for (index, builder) in self.builders.0.iter_mut().enumerate() {
            if builder.payload.is_none() {
                return Err(OutgoingBundleBuilderError::MissingTransactionBuilderField(
                    "payload",
                ));
            } else if builder.address.is_none() {
                return Err(OutgoingBundleBuilderError::MissingTransactionBuilderField(
                    "address",
                ));
            } else if builder.value.is_none() {
                return Err(OutgoingBundleBuilderError::MissingTransactionBuilderField(
                    "value",
                ));
            } else if builder.tag.is_none() {
                return Err(OutgoingBundleBuilderError::MissingTransactionBuilderField(
                    "tag",
                ));
            }

            builder.index.replace(Index(index));
            builder.last_index.replace(Index(last_index));

            // Safe to unwrap since we just checked it's not None
            sum += builder.value.as_ref().unwrap().0;
        }

        if sum != 0 {
            return Err(OutgoingBundleBuilderError::Inconsistent(sum));
        }

        Ok(StagedOutgoingBundleBuilder::<E, OutgoingSealed> {
            builders: self.builders,
            essence_sponge: PhantomData,
            stage: PhantomData,
        })
    }
}

impl<E: Sponge + Default> StagedOutgoingBundleBuilder<E, OutgoingSealed> {
    // TODO TEST
    fn has_no_input(&self) -> Result<(), OutgoingBundleBuilderError> {
        for builder in &self.builders.0 {
            // Safe to unwrap since we made sure it's not None in `seal`
            if builder.value.as_ref().unwrap().0 < 0 {
                return Err(OutgoingBundleBuilderError::UnsignedInput);
            }
        }

        Ok(())
    }

    // TODO TEST
    pub fn attach_local(
        self,
        trunk: Hash,
        branch: Hash,
    ) -> Result<StagedOutgoingBundleBuilder<E, OutgoingAttached>, OutgoingBundleBuilderError> {
        // Checking that no transaction actually needs to be signed (no inputs)
        self.has_no_input()?;

        StagedOutgoingBundleBuilder::<E, OutgoingSigned> {
            builders: self.builders,
            essence_sponge: PhantomData,
            stage: PhantomData,
        }
        .attach_local(trunk, branch)
    }

    // TODO TEST
    pub fn attach_remote(
        self,
        trunk: Hash,
        branch: Hash,
    ) -> Result<StagedOutgoingBundleBuilder<E, OutgoingAttached>, OutgoingBundleBuilderError> {
        // Checking that no transaction actually needs to be signed (no inputs)
        self.has_no_input()?;

        StagedOutgoingBundleBuilder::<E, OutgoingSigned> {
            builders: self.builders,
            essence_sponge: PhantomData,
            stage: PhantomData,
        }
        .attach_remote(trunk, branch)
    }

    // TODO TEST
    pub fn sign(
        self,
    ) -> Result<StagedOutgoingBundleBuilder<E, OutgoingSigned>, OutgoingBundleBuilderError> {
        // TODO Impl
        Ok(StagedOutgoingBundleBuilder::<E, OutgoingSigned> {
            builders: self.builders,
            essence_sponge: PhantomData,
            stage: PhantomData,
        })
    }
}

impl<E: Sponge + Default> StagedOutgoingBundleBuilder<E, OutgoingSigned> {
    // TODO TEST
    pub fn attach_local(
        self,
        trunk: Hash,
        branch: Hash,
    ) -> Result<StagedOutgoingBundleBuilder<E, OutgoingAttached>, OutgoingBundleBuilderError> {
        // TODO Impl
        Ok(StagedOutgoingBundleBuilder::<E, OutgoingAttached> {
            builders: self.builders,
            essence_sponge: PhantomData,
            stage: PhantomData,
        })
    }

    // TODO TEST
    pub fn attach_remote(
        self,
        trunk: Hash,
        branch: Hash,
    ) -> Result<StagedOutgoingBundleBuilder<E, OutgoingAttached>, OutgoingBundleBuilderError> {
        // TODO Impl
        Ok(StagedOutgoingBundleBuilder::<E, OutgoingAttached> {
            builders: self.builders,
            essence_sponge: PhantomData,
            stage: PhantomData,
        })
    }
}

impl<E: Sponge + Default> StagedOutgoingBundleBuilder<E, OutgoingAttached> {
    // TODO TEST
    pub fn build(self) -> Result<Bundle, OutgoingBundleBuilderError> {
        let mut transactions = Transactions::new();

        for transaction_builder in self.builders.0 {
            transactions.push(
                transaction_builder
                    .build()
                    .map_err(|e| OutgoingBundleBuilderError::FailedTransactionBuild(e))?,
            );
        }

        Ok(Bundle(transactions))
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::transaction::{Address, Nonce, Payload, Tag, Timestamp, Value};

    fn default_transaction_builder() -> TransactionBuilder {
        TransactionBuilder::new()
            .with_payload(Payload::zeros())
            .with_address(Address::zeros())
            .with_value(Value(0))
            .with_obsolete_tag(Tag::zeros())
            .with_timestamp(Timestamp(0))
            .with_index(Index(0))
            .with_last_index(Index(0))
            .with_tag(Tag::zeros())
            .with_attachment_ts(Timestamp(0))
            .with_bundle(Hash::zeros())
            .with_trunk(Hash::zeros())
            .with_branch(Hash::zeros())
            .with_attachment_lbts(Timestamp(0))
            .with_attachment_ubts(Timestamp(0))
            .with_nonce(Nonce::zeros())
    }

    /// Bundle

    /// Incoming bundle builder

    #[test]
    fn incoming_bundle_builder_test() -> Result<(), IncomingBundleBuilderError> {
        let mut bundle_builder = IncomingBundleBuilder::new();

        for _ in 0..5 {
            bundle_builder.push(default_transaction_builder().build().unwrap());
        }

        let bundle = bundle_builder.validate()?.build();

        assert_eq!(bundle.len(), 5);

        Ok(())
    }

    /// Outgoing bundle builder

    // TODO Also check to attach if value ?
    #[test]
    fn outgoing_bundle_builder_value_test() -> Result<(), OutgoingBundleBuilderError> {
        let mut bundle_builder = OutgoingBundleBuilder::new();

        for _ in 0..3 {
            bundle_builder.push(default_transaction_builder());
        }

        let bundle = bundle_builder
            .seal()?
            .sign()?
            .attach_local(Hash::zeros(), Hash::zeros())?
            .build()?;

        assert_eq!(bundle.len(), 3);

        Ok(())
    }

    // TODO Also check to sign if data ?
    #[test]
    fn outgoing_bundle_builder_data_test() -> Result<(), OutgoingBundleBuilderError> {
        let mut bundle_builder = OutgoingBundleBuilder::new();

        for _ in 0..3 {
            bundle_builder.push(default_transaction_builder());
        }

        let bundle = bundle_builder
            .seal()?
            .attach_local(Hash::zeros(), Hash::zeros())?
            .build()?;

        assert_eq!(bundle.len(), 3);

        Ok(())
    }
}

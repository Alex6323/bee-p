use crate::bundle::Bundle;
use crate::constants::IOTA_SUPPLY;
use crate::transaction::{Hash, Index, Transaction, TransactionBuilder, Transactions};

use bee_crypto::Sponge;
use bee_signing::{PublicKey, Signature, WotsPublicKey};
use bee_ternary::TritBuf;

use std::marker::PhantomData;

#[derive(Debug)]
pub enum IncomingBundleBuilderError {
    Empty,
    InvalidIndex(usize),
    InvalidLastIndex(usize),
    InvalidValue(i64),
    InvalidSignature,
}

pub trait IncomingBundleBuilderStage {}

pub struct IncomingRaw;
impl IncomingBundleBuilderStage for IncomingRaw {}

pub struct IncomingValidated;
impl IncomingBundleBuilderStage for IncomingValidated {}

pub struct StagedIncomingBundleBuilder<E, P, S> {
    transactions: Transactions,
    essence_sponge: PhantomData<E>,
    public_key: PhantomData<P>,
    stage: PhantomData<S>,
}

// TODO default kerl
pub type IncomingBundleBuilder =
    StagedIncomingBundleBuilder<bee_crypto::CurlP81, WotsPublicKey<bee_crypto::CurlP81>, IncomingRaw>;

impl<E, P> StagedIncomingBundleBuilder<E, P, IncomingRaw>
where
    E: Sponge + Default,
    P: PublicKey,
{
    // TODO TEST
    pub fn new() -> Self {
        Self {
            transactions: Transactions::new(),
            essence_sponge: PhantomData,
            public_key: PhantomData,
            stage: PhantomData,
        }
    }

    // TODO TEST
    pub fn push(&mut self, transaction: Transaction) {
        self.transactions.push(transaction);
    }

    // TODO TEST
    // TODO common with outgoing bundle builder
    fn calculate_hash(&self) -> TritBuf {
        // TODO Impl
        let mut sponge = E::default();

        for builder in &self.transactions.0 {
            // sponge.absorb(builder.address.0);
        }

        sponge.squeeze()
    }

    fn validate_signatures(&self) -> Result<(), IncomingBundleBuilderError> {
        // TODO get real values
        let public_key = P::from_buf(TritBuf::new());
        let signature = P::Signature::from_buf(TritBuf::new());

        match public_key.verify(&[], &signature) {
            Ok(valid) => match valid {
                true => Ok(()),
                false => Err(IncomingBundleBuilderError::InvalidSignature),
            },
            Err(_) => Err(IncomingBundleBuilderError::InvalidSignature),
        }
    }

    // TODO TEST
    // TODO make it parameterized ?
    pub fn validate(self) -> Result<StagedIncomingBundleBuilder<E, P, IncomingValidated>, IncomingBundleBuilderError> {
        let mut sum: i64 = 0;

        if self.transactions.len() == 0 {
            return Err(IncomingBundleBuilderError::Empty);
        }

        let last_index = self.transactions.len() - 1;

        for (index, transaction) in self.transactions.0.iter().enumerate() {
            if index != transaction.index().0 {
                return Err(IncomingBundleBuilderError::InvalidIndex(transaction.index().0));
            }

            if last_index != transaction.last_index().0 {
                return Err(IncomingBundleBuilderError::InvalidLastIndex(transaction.last_index().0));
            }

            sum += transaction.value.0;
            if sum.abs() > IOTA_SUPPLY {
                return Err(IncomingBundleBuilderError::InvalidValue(sum));
            }
        }

        if sum != 0 {
            return Err(IncomingBundleBuilderError::InvalidValue(sum));
        }

        // TODO check last trit of address
        // TODO check bundle hash
        // TODO check signatures
        // TODO check trunk chaining
        // TODO check trunk/branch consistency
        // TODO check trunk/branch are tails
        // TODO ontology ?

        self.validate_signatures()?;

        Ok(StagedIncomingBundleBuilder::<E, P, IncomingValidated> {
            transactions: self.transactions,
            essence_sponge: PhantomData,
            public_key: PhantomData,
            stage: PhantomData,
        })
    }
}

impl<E, P> StagedIncomingBundleBuilder<E, P, IncomingValidated>
where
    E: Sponge + Default,
    P: PublicKey,
{
    // TODO TEST
    pub fn build(self) -> Bundle {
        Bundle(self.transactions)
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::transaction::{Address, Nonce, Payload, Tag, Timestamp, Value};

    fn default_transaction_builder(index: usize, last_index: usize) -> TransactionBuilder {
        TransactionBuilder::new()
            .with_payload(Payload::zeros())
            .with_address(Address::zeros())
            .with_value(Value(0))
            .with_obsolete_tag(Tag::zeros())
            .with_timestamp(Timestamp(0))
            .with_index(Index(index))
            .with_last_index(Index(last_index))
            .with_tag(Tag::zeros())
            .with_attachment_ts(Timestamp(0))
            .with_bundle(Hash::zeros())
            .with_trunk(Hash::zeros())
            .with_branch(Hash::zeros())
            .with_attachment_lbts(Timestamp(0))
            .with_attachment_ubts(Timestamp(0))
            .with_nonce(Nonce::zeros())
    }

    #[test]
    fn incoming_bundle_builder_test() -> Result<(), IncomingBundleBuilderError> {
        let bundle_size = 3;
        let mut bundle_builder = IncomingBundleBuilder::new();

        for i in 0..bundle_size {
            bundle_builder.push(default_transaction_builder(i, bundle_size - 1).build().unwrap());
        }

        let bundle = bundle_builder.validate()?.build();

        assert_eq!(bundle.len(), bundle_size);

        Ok(())
    }
}

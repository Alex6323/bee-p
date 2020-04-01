use crate::{
    bundle::{
        Bundle,
        Transactions,
    },
    constants::{
        ADDRESS,
        IOTA_SUPPLY,
    },
    transaction::{
        Transaction,
        TransactionField,
    },
};

use bee_crypto::{
    Kerl,
    Sponge,
};
use bee_signing::{
    PublicKey,
    Signature,
    WotsPublicKey,
};
use bee_ternary::{
    trit::Btrit,
    TritBuf,
};

use std::marker::PhantomData;

#[derive(Debug)]
pub enum IncomingBundleBuilderError {
    Empty,
    InvalidIndex(usize),
    InvalidLastIndex(usize),
    InvalidValue(i64),
    InvalidSignature,
    InvalidAddress,
    InvalidBundleHash,
    InvalidBranchInconsistency,
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

pub type IncomingBundleBuilder = StagedIncomingBundleBuilder<Kerl, WotsPublicKey<Kerl>, IncomingRaw>;

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

        for _builder in &self.transactions.0 {
            // sponge.absorb(builder.address.0);
        }

        sponge
            .squeeze()
            .unwrap_or_else(|_| panic!("Panicked when unwrapping the sponge hash function."))
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
            Err(IncomingBundleBuilderError::Empty)?;
        }

        let last_index = self.transactions.len() - 1;

        let bundle_hash_calculated = self.calculate_hash().as_i8_slice().to_vec();

        let first_branch = self.transactions.0[0].branch();

        // TODO - check trunk of the last transaction and branch is tail, the same tail

        for (index, transaction) in self.transactions.0.iter().enumerate() {
            if index != *transaction.index().to_inner() {
                Err(IncomingBundleBuilderError::InvalidIndex(
                    *transaction.index().to_inner(),
                ))?;
            }

            if last_index != *transaction.last_index().to_inner() {
                Err(IncomingBundleBuilderError::InvalidLastIndex(
                    *transaction.last_index().to_inner(),
                ))?;
            }

            sum += *transaction.value.to_inner();
            if sum.abs() > IOTA_SUPPLY {
                Err(IncomingBundleBuilderError::InvalidValue(sum))?;
            }

            if *transaction.value.to_inner() != 0
                && transaction
                    .address()
                    .to_inner()
                    .get(ADDRESS.trit_offset.length - 1)
                    .unwrap()
                    != Btrit::Zero
            {
                Err(IncomingBundleBuilderError::InvalidAddress)?;
            }

            if index == 0 as usize && bundle_hash_calculated.ne(&transaction.bundle().to_inner().as_i8_slice().to_vec())
            {
                Err(IncomingBundleBuilderError::InvalidBundleHash)?;
            }

            if index > 0 as usize && transaction.branch().ne(first_branch) {
                Err(IncomingBundleBuilderError::InvalidBranchInconsistency)?;
            }

            // TODO - for each transaction's hash check that it is its prev trunk
        }

        if sum != 0 {
            Err(IncomingBundleBuilderError::InvalidValue(sum))?;
        }

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
    use crate::transaction::{
        Address,
        Hash,
        Index,
        Nonce,
        Payload,
        Tag,
        Timestamp,
        TransactionBuilder,
        Value,
    };

    fn default_transaction_builder(index: usize, last_index: usize) -> TransactionBuilder {
        TransactionBuilder::new()
            .with_payload(Payload::zeros())
            .with_address(Address::zeros())
            .with_value(Value::from_inner_unchecked(0))
            .with_obsolete_tag(Tag::zeros())
            .with_timestamp(Timestamp::from_inner_unchecked(0))
            .with_index(Index::from_inner_unchecked(index))
            .with_last_index(Index::from_inner_unchecked(last_index))
            .with_tag(Tag::zeros())
            .with_attachment_ts(Timestamp::from_inner_unchecked(0))
            .with_bundle(Hash::zeros())
            .with_trunk(Hash::zeros())
            .with_branch(Hash::zeros())
            .with_attachment_lbts(Timestamp::from_inner_unchecked(0))
            .with_attachment_ubts(Timestamp::from_inner_unchecked(0))
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

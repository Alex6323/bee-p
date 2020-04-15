use crate::{
    bundle::Bundle,
    constants::IOTA_SUPPLY,
    transaction::{
        Hash,
        Index,
        Tag,
        TransactionBuilder,
        TransactionBuilders,
        TransactionError,
        TransactionField,
        Transactions,
    },
};

use bee_crypto::{
    Kerl,
    Sponge,
};
use bee_ternary::Btrit;
use bee_signing::normalize_hash;

use std::marker::PhantomData;

#[derive(Debug)]
pub enum OutgoingBundleBuilderError {
    Empty,
    UnsignedInput,
    InvalidValue(i64),
    MissingTransactionBuilderField(&'static str),
    TransactionError(TransactionError),
}

pub trait OutgoingBundleBuilderStage {}

pub struct OutgoingRaw;
impl OutgoingBundleBuilderStage for OutgoingRaw {}

pub struct OutgoingSealed;
impl OutgoingBundleBuilderStage for OutgoingSealed {}

pub struct OutgoingSigned;
impl OutgoingBundleBuilderStage for OutgoingSigned {}

pub struct OutgoingAttached;
impl OutgoingBundleBuilderStage for OutgoingAttached {}

pub struct StagedOutgoingBundleBuilder<E, S> {
    builders: TransactionBuilders,
    essence_sponge: PhantomData<E>,
    stage: PhantomData<S>,
}

// TODO default to Kerl
pub type OutgoingBundleBuilder = StagedOutgoingBundleBuilder<Kerl, OutgoingRaw>;

impl<E, S> StagedOutgoingBundleBuilder<E, S>
where
    E: Sponge + Default,
    S: OutgoingBundleBuilderStage,
{
    // TODO TEST
    fn set_bundle_hash(&mut self) -> Result<(), OutgoingBundleBuilderError> {
        let mut sponge = E::default();
        let mut tag = match self.builders.0.get(0) {
            Some(builder) => match builder.obsolete_tag.clone() {
                Some(tag) => tag.to_inner().to_owned(),
                _ => {
                    return Err(OutgoingBundleBuilderError::MissingTransactionBuilderField(
                        "Missing obsolete tag",
                    ))
                }
            },
            _ => return Err(OutgoingBundleBuilderError::Empty),
        };

        let hash = loop {
            sponge.reset();

            for builder in &self.builders.0 {
                let _ = sponge.absorb(&builder.essence());
            }

            let hash = sponge
                .squeeze()
                .unwrap_or_else(|_| panic!("Panicked when unwrapping the sponge hash function."));

            let hash = normalize_hash(&hash);
            let mut has_m_bug = false;
            for trits in hash.chunks(3) {
                let mut is_m = true;

                for trit in trits.iter() {
                    if *trit != Btrit::PlusOne {
                        is_m = false;
                        break;
                    }
                }

                if is_m {
                    has_m_bug = true;
                    break;
                }
            }

            if !has_m_bug {
                break Hash::from_inner_unchecked(hash);
            } else {
                // obsolete_tag + 1
                for i in 0..tag.len() {
                    // Safe to unwrap since it's in the rage of tag
                    match tag.get(i).unwrap() {
                        Btrit::NegOne => {
                            tag.set(i, Btrit::Zero);
                            break;
                        },
                        Btrit::Zero => {
                            tag.set(i, Btrit::PlusOne);
                            break;
                        },
                        Btrit::PlusOne => tag.set(i, Btrit::NegOne),
                    };
                }
                // Safe to unwrap because we already check first tx exists.
                self.builders.0.get_mut(0).unwrap().tag = Some(Tag::from_inner_unchecked(tag.clone()));
            }
        };

        for builder in &mut self.builders.0 {
            builder.obsolete_tag = Some(Tag::from_inner_unchecked(tag.clone()));
            builder.bundle = Some(hash.clone());
        }

        Ok(())
    }
}

impl<E: Sponge + Default> StagedOutgoingBundleBuilder<E, OutgoingRaw> {
    // TODO TEST
    pub fn new() -> Self {
        Self {
            builders: TransactionBuilders::default(),
            essence_sponge: PhantomData,
            stage: PhantomData,
        }
    }

    // TODO TEST
    pub fn push(&mut self, builder: TransactionBuilder) {
        self.builders.push(builder);
    }

    // TODO TEST
    pub fn seal(mut self) -> Result<StagedOutgoingBundleBuilder<E, OutgoingSealed>, OutgoingBundleBuilderError> {
        // TODO Impl
        // TODO should call validate() on transaction builders ?
        let mut sum: i64 = 0;
        let last_index = self.builders.len() - 1;

        if self.builders.len() == 0 {
            Err(OutgoingBundleBuilderError::Empty)?;
        }

        for (index, builder) in self.builders.0.iter_mut().enumerate() {
            if builder.payload.is_none() {
                Err(OutgoingBundleBuilderError::MissingTransactionBuilderField("payload"))?;
            } else if builder.address.is_none() {
                Err(OutgoingBundleBuilderError::MissingTransactionBuilderField("address"))?;
            } else if builder.value.is_none() {
                Err(OutgoingBundleBuilderError::MissingTransactionBuilderField("value"))?;
            } else if builder.tag.is_none() {
                Err(OutgoingBundleBuilderError::MissingTransactionBuilderField("tag"))?;
            }

            builder.index.replace(Index::from_inner_unchecked(index));
            builder.last_index.replace(Index::from_inner_unchecked(last_index));

            // Safe to unwrap since we just checked it's not None
            sum += builder.value.as_ref().unwrap().to_inner();
            if sum.abs() > IOTA_SUPPLY {
                Err(OutgoingBundleBuilderError::InvalidValue(sum))?;
            }
        }

        if sum != 0 {
            Err(OutgoingBundleBuilderError::InvalidValue(sum))?;
        }

        self.set_bundle_hash()?;

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
            if *builder.value.as_ref().unwrap().to_inner() < 0 {
                Err(OutgoingBundleBuilderError::UnsignedInput)?;
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
    pub fn sign(self) -> Result<StagedOutgoingBundleBuilder<E, OutgoingSigned>, OutgoingBundleBuilderError> {

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
        _trunk: Hash,
        _branch: Hash,
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
        _trunk: Hash,
        _branch: Hash,
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
                    .map_err(|e| OutgoingBundleBuilderError::TransactionError(e))?,
            );
        }

        Ok(Bundle(transactions))
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::transaction::{
        Address,
        Nonce,
        Payload,
        Tag,
        Timestamp,
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

    // TODO Also check to attach if value ?
    #[test]
    fn outgoing_bundle_builder_value_test() -> Result<(), OutgoingBundleBuilderError> {
        let bundle_size = 3;
        let mut bundle_builder = OutgoingBundleBuilder::new();

        for i in 0..bundle_size {
            bundle_builder.push(default_transaction_builder(i, bundle_size - 1));
        }

        let bundle = bundle_builder
            .seal()?
            .sign()?
            .attach_local(Hash::zeros(), Hash::zeros())?
            .build()?;

        assert_eq!(bundle.len(), bundle_size);

        Ok(())
    }

    // TODO Also check to sign if data ?
    #[test]
    fn outgoing_bundle_builder_data_test() -> Result<(), OutgoingBundleBuilderError> {
        let bundle_size = 3;
        let mut bundle_builder = OutgoingBundleBuilder::new();

        for i in 0..bundle_size {
            bundle_builder.push(default_transaction_builder(i, bundle_size - 1));
        }

        let bundle = bundle_builder
            .seal()?
            .attach_local(Hash::zeros(), Hash::zeros())?
            .build()?;

        assert_eq!(bundle.len(), bundle_size);

        Ok(())
    }
}

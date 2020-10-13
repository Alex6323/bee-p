// Copyright 2020 IOTA Stiftung
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file except in compliance with
// the License. You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software distributed under the License is distributed on
// an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and limitations under the License.

mod constants;
mod essence;
mod input;
mod output;
mod transaction_id;
mod unlock;

use crate::{payload::Payload, Error};

use constants::{INPUT_OUTPUT_COUNT_RANGE, INPUT_OUTPUT_INDEX_RANGE};

pub use essence::TransactionEssence;
pub use input::{Input, UTXOInput};
pub use output::{Address, Ed25519Address, Output, SignatureLockedSingleOutput, WotsAddress};
pub use transaction_id::TransactionId;
pub use unlock::{Ed25519Signature, ReferenceUnlock, SignatureUnlock, UnlockBlock, WotsSignature};

use bee_common_ext::packable::{Error as PackableError, Packable, Read, Write};
pub use bee_signing_ext::Seed;
use bee_signing_ext::{
    binary::{BIP32Path, Ed25519PrivateKey, Ed25519PublicKey, Ed25519Signature as Ed25Signature},
    Signature as SignatureTrait, Signer, Verifier,
};

use serde::{Deserialize, Serialize};

use alloc::{boxed::Box, vec::Vec};
use core::{cmp::Ordering, slice::Iter};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Transaction {
    pub essence: TransactionEssence,
    // TODO Box
    pub unlock_blocks: Vec<UnlockBlock>,
}

impl Packable for Transaction {
    fn packed_len(&self) -> usize {
        self.essence.packed_len()
            + 0u16.packed_len()
            + self.unlock_blocks.iter().map(|block| block.packed_len()).sum::<usize>()
    }

    fn pack<W: Write>(&self, buf: &mut W) -> Result<(), PackableError> {
        self.essence.pack(buf)?;

        (self.unlock_blocks.len() as u16).pack(buf)?;
        for unlock_block in &self.unlock_blocks {
            unlock_block.pack(buf)?;
        }

        Ok(())
    }

    fn unpack<R: Read>(buf: &mut R) -> Result<Self, PackableError>
    where
        Self: Sized,
    {
        let essence = TransactionEssence::unpack(buf)?;

        let unlock_blocks_len = u16::unpack(buf)? as usize;
        let mut unlock_blocks = Vec::with_capacity(unlock_blocks_len);
        for _ in 0..unlock_blocks_len {
            unlock_blocks.push(UnlockBlock::unpack(buf)?);
        }

        Ok(Self { essence, unlock_blocks })
    }
}

impl Transaction {
    pub fn builder(seed: &Seed) -> TransactionBuilder {
        TransactionBuilder::new(seed)
    }

    pub fn validate(&self) -> Result<(), Error> {
        // Should we add this field? -> Transaction Type value must be 0, denoting an Unsigned Transaction

        // Inputs validation
        let transaction = &self.essence;
        // Inputs Count must be 0 < x <= 127
        if !INPUT_OUTPUT_COUNT_RANGE.contains(&transaction.inputs().len()) {
            return Err(Error::CountError);
        }

        // At least one input must be specified
        if transaction.inputs().is_empty() {
            return Err(Error::NoInput);
        }

        for i in transaction.inputs().iter() {
            // Input Type value must be 0, denoting an UTXO Input.
            match i {
                Input::UTXO(u) => {
                    // Transaction Output Index must be 0 ≤ x < 127
                    if !INPUT_OUTPUT_INDEX_RANGE.contains(&u.index()) {
                        return Err(Error::CountError);
                    }

                    // Every combination of Transaction ID + Transaction Output Index must be unique in the inputs set.
                    if transaction.inputs().iter().filter(|j| *j == i).count() > 1 {
                        return Err(Error::DuplicateError);
                    }
                }
            }
        }

        // Inputs must be in lexicographical order of their serialized form.
        // TODO
        // if !is_sorted(transaction.inputs.iter()) {
        //     return Err(Error::OrderError);
        // }

        // Output validation
        // Outputs Count must be 0 < x <= 127
        if !INPUT_OUTPUT_COUNT_RANGE.contains(&transaction.outputs().len()) {
            return Err(Error::CountError);
        }

        // At least one output must be specified
        if transaction.outputs().is_empty() {
            return Err(Error::NoOutput);
        }

        let mut total = 0;
        for i in transaction.outputs().iter() {
            // Output Type must be 0, denoting a SignatureLockedSingle.
            match i {
                output::Output::SignatureLockedSingle(u) => {
                    // Address Type must either be 0 or 1, denoting a WOTS- or Ed25519 address.

                    // If Address is of type WOTS address, its bytes must be valid T5B1 bytes.

                    // The Address must be unique in the set of SigLockedSingleDeposits
                    if transaction
                        .outputs()
                        .iter()
                        .filter(|j| match *j {
                            output::Output::SignatureLockedSingle(s) => s.address() == u.address(),
                        })
                        .count()
                        > 1
                    {
                        return Err(Error::DuplicateError);
                    }

                    // Amount must be > 0
                    let amount = u.amount().get();
                    if amount == 0 {
                        return Err(Error::AmountError);
                    }

                    total += amount;
                }
            }
        }

        // Outputs must be in lexicographical order by their serialized form
        // TODO
        // if !is_sorted(transaction.outputs.iter()) {
        //     return Err(Error::OrderError);
        // }

        // Accumulated output balance must not exceed the total supply of tokens 2'779'530'283'277'761
        if total > 2779530283277761 {
            return Err(Error::AmountError);
        }

        // Payload Length must be 0 (to indicate that there's no payload) or be valid for the specified payload type.
        // Payload Type must be one of the supported payload types if Payload Length is not 0.

        // Unlock Blocks validation
        // Unlock Blocks Count must match the amount of inputs. Must be 0 < x < 127.
        if !INPUT_OUTPUT_COUNT_RANGE.contains(&self.unlock_blocks.len()) {
            return Err(Error::CountError);
        }

        for (i, block) in self.unlock_blocks.iter().enumerate() {
            // Signature Unlock Blocks must define either an Ed25519- or WOTS Signature
            match block {
                UnlockBlock::Reference(r) => {
                    // Reference Unlock Blocks must specify a previous Unlock Block which is not of type Reference
                    // Unlock Block. Since it's not the first input it unlocks, it must have
                    // differente transaction id from previous one
                    if i != 0 {
                        match &transaction.inputs()[i] {
                            Input::UTXO(u) => match &transaction.inputs()[i - 1] {
                                Input::UTXO(v) => {
                                    if u.id() != v.id() {
                                        return Err(Error::InvalidIndex);
                                    }
                                }
                            },
                        }
                    }

                    // The reference index must therefore be < the index of the Reference Unlock Block
                    if r.index() >= i as u16 {
                        return Err(Error::InvalidIndex);
                    }
                }
                UnlockBlock::Signature(s) => {
                    // A Signature Unlock Block unlocking multiple inputs must only appear once (be unique) and be
                    // positioned at same index of the first input it unlocks.
                    if self.unlock_blocks.iter().filter(|j| *j == block).count() > 1 {
                        return Err(Error::DuplicateError);
                    }

                    // Since it's first input it unlocks, it must have differente transaction id from previous one
                    if i != 0 {
                        match &transaction.inputs()[i] {
                            Input::UTXO(u) => match &transaction.inputs()[i - 1] {
                                Input::UTXO(v) => {
                                    if u.id() == v.id() {
                                        return Err(Error::InvalidIndex);
                                    }
                                }
                            },
                        }
                    }

                    // Semantic Validation: The Signature Unlock Blocks are valid, i.e. the signatures prove ownership
                    // over the addresses of the referenced UTXOs.
                    // let serialized_inputs = bincode::serialize(&transaction.inputs[i]).map_err(|_|
                    // Error::HashError)?; TODO
                    let serialized_inputs = [];
                    match s {
                        SignatureUnlock::Ed25519(sig) => {
                            let key = Ed25519PublicKey::from_bytes(sig.public_key())?;
                            let signature = Ed25Signature::from_bytes(sig.signature())?;
                            key.verify(&serialized_inputs, &signature)?;
                        }
                        SignatureUnlock::Wots(_) => {}
                    }
                }
            }
        }

        // TODO Semantic Validation
        // TODO The UTXOs the transaction references must be known (booked) and unspent.
        // TODO The transaction is spending the entirety of the funds of the referenced UTXOs to the outputs.

        Ok(())
    }
}

#[allow(dead_code)]
fn is_sorted<T: Ord>(iterator: Iter<T>) -> bool {
    let mut iterator = iterator;
    let mut last = match iterator.next() {
        Some(e) => e,
        None => return true,
    };

    for curr in iterator {
        if let Ordering::Greater = &last.cmp(&curr) {
            return false;
        }
        last = curr;
    }

    true
}

pub struct TransactionBuilder<'a> {
    seed: &'a Seed,
    inputs: Vec<(Input, BIP32Path)>,
    outputs: Vec<Output>,
    payload: Option<Payload>,
}

impl<'a> TransactionBuilder<'a> {
    pub fn new(seed: &'a Seed) -> Self {
        Self {
            seed,
            inputs: Vec::new(),
            outputs: Vec::new(),
            payload: None,
        }
    }

    pub fn set_inputs(mut self, mut inputs: Vec<(Input, BIP32Path)>) -> Self {
        self.inputs.append(&mut inputs);

        self
    }

    pub fn set_outputs(mut self, outputs: Vec<Output>) -> Self {
        let mut outputs = outputs;
        self.outputs.append(&mut outputs);

        self
    }

    pub fn set_payload(mut self, payload: Payload) -> Self {
        self.payload = Some(payload);

        self
    }

    pub fn build(self) -> Result<Transaction, Error> {
        let inputs = self.inputs;
        let outputs = self.outputs;

        if inputs.is_empty() || outputs.is_empty() {
            return Err(Error::CountError);
        }

        // TODO
        // inputs.sort();
        // outputs.sort();

        let mut unlock_blocks = Vec::new();
        let mut last_index = (None, -1);
        for (_i, path) in &inputs {
            if last_index.0 == Some(path) {
                // TODO justify unwrap
                unlock_blocks.push(UnlockBlock::Reference(
                    ReferenceUnlock::new(last_index.1 as u16).unwrap(),
                ));
            } else {
                let serialized_inputs = [];
                // TODO
                // let serialized_inputs = bincode::serialize(i).map_err(|_| Error::HashError)?;
                match &self.seed {
                    Seed::Ed25519(s) => {
                        let private_key = Ed25519PrivateKey::generate_from_seed(s, &path)?;
                        let public_key = private_key.generate_public_key().to_bytes();
                        let signature = Box::new(private_key.sign(&serialized_inputs).to_bytes());
                        unlock_blocks.push(UnlockBlock::Signature(SignatureUnlock::Ed25519(Ed25519Signature::new(
                            public_key, signature,
                        ))));
                    }
                    Seed::Wots(_) => {
                        // let private_key = WotsShakePrivateKeyGeneratorBuilder::<Kerl>::default()
                        //     .with_security_level(WotsSecurityLevel::Medium)
                        //     .build()
                        //     .map_err(|_| Error::HashError)?
                        //     .generate_from_seed(s, *index)
                        //     .map_err(|_| Error::HashError)?;
                        // TODO create signature
                    }
                }

                last_index = (Some(path), (unlock_blocks.len() - 1) as isize);
            }
        }

        let inputs: Box<[Input]> = inputs
            .into_iter()
            .map(|(i, _)| i)
            .collect::<Vec<Input>>()
            .into_boxed_slice();

        // TODO use TransactionEssenceBuilder
        Ok(Transaction {
            essence: TransactionEssence {
                inputs,
                outputs: outputs.into_boxed_slice(),
                payload: self.payload,
            },
            unlock_blocks,
        })
    }
}

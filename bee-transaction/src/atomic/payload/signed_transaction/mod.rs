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

mod input;
mod output;
mod unlock;
mod unsigned_transaction;

use crate::atomic::{payload::Payload, Error};

pub use bee_signing_ext::Seed;
pub use input::{Input, UTXOInput};
pub use output::{Address, Ed25519Address, Output, SigLockedSingleDeposit, WotsAddress};
pub use unlock::{Ed25519Signature, ReferenceUnlock, SignatureUnlock, UnlockBlock, WotsSignature};
pub use unsigned_transaction::UnsignedTransaction;

use bee_signing_ext::{
    binary::{BIP32Path, Ed25519PrivateKey, Ed25519PublicKey, Ed25519Signature as Ed25Signature},
    Signature as SignatureTrait, Signer, Verifier,
};

use serde::{Deserialize, Serialize};

use alloc::vec::Vec;
use core::{cmp::Ordering, slice::Iter};

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct SignedTransaction {
    pub unsigned_transaction: UnsignedTransaction,
    pub unlock_block_count: u8,
    pub unlock_blocks: Vec<UnlockBlock>,
}

impl SignedTransaction {
    pub fn builder(seed: &Seed) -> SignedTransactionBuilder {
        SignedTransactionBuilder::new(seed)
    }

    pub fn validate(&self) -> Result<(), Error> {
        // Should we add this field? -> Transaction Type value must be 0, denoting an Unsigned Transaction

        // Inputs validation
        let transaction = &self.unsigned_transaction;
        // Inputs Count must be 0 < x < 127
        match transaction.inputs.len() {
            1..=126 => (),
            _ => return Err(Error::CountError),
        }

        // At least one input must be specified
        if transaction.inputs.is_empty() {
            return Err(Error::EmptyError);
        }

        let mut combination = Vec::new();
        for i in transaction.inputs.iter() {
            // Input Type value must be 0, denoting an UTXO Input.
            match i {
                Input::UTXO(u) => {
                    // Transaction Output Index must be 0 ≤ x < 127
                    match u.index() {
                        0..=126 => (),
                        _ => return Err(Error::CountError),
                    }

                    // Every combination of Transaction ID + Transaction Output Index must be unique in the inputs set.
                    if !insert_combination(&mut combination, u) {
                        return Err(Error::DuplicateError);
                    }
                }
            }
        }

        // Inputs must be in lexicographical order of their serialized form.
        if !is_sorted(transaction.inputs.iter()) {
            return Err(Error::OrderError);
        }

        // Output validation
        // Outputs Count must be 0 < x < 127
        match transaction.outputs.len() {
            1..=126 => (),
            _ => return Err(Error::CountError),
        }

        // At least one output must be specified
        if transaction.outputs.is_empty() {
            return Err(Error::EmptyError);
        }

        let mut total = 0;
        let mut combination = Vec::new();
        for i in transaction.outputs.iter() {
            // Output Type must be 0, denoting a SigLockedSingleDeposit.
            match i {
                output::Output::SigLockedSingleDeposit(u) => {
                    // Address Type must either be 0 or 1, denoting a WOTS- or Ed25519 address.

                    // If Address is of type WOTS address, its bytes must be valid T5B1 bytes.

                    // The Address must be unique in the set of SigLockedSingleDeposits
                    if !insert_combination(&mut combination, u.address()) {
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
        if !is_sorted(transaction.outputs.iter()) {
            return Err(Error::OrderError);
        }

        // Accumulated output balance must not exceed the total supply of tokens 2'779'530'283'277'761
        if total > 2779530283277761 {
            return Err(Error::AmountError);
        }

        // Payload Length must be 0 (to indicate that there's no payload) or be valid for the specified payload type.
        // Payload Type must be one of the supported payload types if Payload Length is not 0.

        // Unlock Blocks validation
        // Unlock Blocks Count must match the amount of inputs. Must be 0 < x < 127.
        match self.unlock_block_count {
            1..=126 => (),
            _ => return Err(Error::CountError),
        }

        // Unlock Block Type must either be 0 or 1, denoting a Signature Unlock Block or Reference Unlock block.
        let mut combination = Vec::new();
        for (i, block) in self.unlock_blocks.iter().enumerate() {
            // Signature Unlock Blocks must define either an Ed25519- or WOTS Signature
            match block {
                UnlockBlock::Reference(r) => {
                    // Reference Unlock Blocks must specify a previous Unlock Block which is not of type Reference
                    // Unlock Block. Since it's not the first input it unlocks, it must have
                    // differente transaction id from previous one
                    if i != 0 {
                        match &transaction.inputs[i] {
                            Input::UTXO(u) => match &transaction.inputs[i - 1] {
                                Input::UTXO(v) => {
                                    if u.id() != v.id() {
                                        return Err(Error::IndexError);
                                    }
                                }
                            },
                        }
                    }

                    // The reference index must therefore be < the index of the Reference Unlock Block
                    if r.index() >= i as u8 {
                        return Err(Error::IndexError);
                    }
                }
                UnlockBlock::Signature(s) => {
                    // A Signature Unlock Block unlocking multiple inputs must only appear once (be unique) and be
                    // positioned at same index of the first input it unlocks.
                    if !insert_combination(&mut combination, s) {
                        return Err(Error::DuplicateError);
                    }

                    // Since it's first input it unlocks, it must have differente transaction id from previous one
                    if i != 0 {
                        match &transaction.inputs[i] {
                            Input::UTXO(u) => match &transaction.inputs[i - 1] {
                                Input::UTXO(v) => {
                                    if u.id() == v.id() {
                                        return Err(Error::IndexError);
                                    }
                                }
                            },
                        }
                    }

                    // Semantic Validation: The Signature Unlock Blocks are valid, i.e. the signatures prove ownership
                    // over the addresses of the referenced UTXOs.
                    let serialized_inputs = bincode::serialize(&transaction.inputs[i]).map_err(|_| Error::HashError)?;
                    match s {
                        SignatureUnlock::Ed25519(sig) => {
                            let key = Ed25519PublicKey::from_bytes(&sig.public_key)?;
                            let signature = Ed25Signature::from_bytes(&sig.signature)?;
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

fn insert_combination<T: PartialEq>(combination: &mut Vec<T>, element: T) -> bool {
    let res = combination.iter().fold(true, |mut acc, i| {
        if i == &element {
            acc = false;
        }
        acc
    });
    if res {
        combination.push(element);
    }
    res
}

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

pub struct SignedTransactionBuilder<'a> {
    seed: &'a Seed,
    inputs: Vec<(Input, BIP32Path)>,
    outputs: Vec<Output>,
    payload: Option<Vec<Payload>>,
}

impl<'a> SignedTransactionBuilder<'a> {
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

    pub fn set_payload(mut self, payload: Vec<Payload>) -> Self {
        self.payload = Some(payload);

        self
    }

    pub fn build(self) -> Result<SignedTransaction, Error> {
        let mut inputs = self.inputs;
        let mut outputs = self.outputs;
        if inputs.is_empty() || outputs.is_empty() {
            return Err(Error::CountError);
        }
        inputs.sort();
        outputs.sort();

        let mut unlock_blocks = Vec::new();
        let mut last_index = (None, -1);
        for (i, path) in &inputs {
            if last_index.0 == Some(path) {
                unlock_blocks.push(UnlockBlock::Reference(ReferenceUnlock::new(last_index.1 as u8)));
            } else {
                let serialized_inputs = bincode::serialize(i).map_err(|_| Error::HashError)?;
                match &self.seed {
                    Seed::Ed25519(s) => {
                        let private_key = Ed25519PrivateKey::generate_from_seed(s, &path)?;
                        let public_key = private_key.generate_public_key().to_bytes();
                        let signature = private_key.sign(&serialized_inputs).to_bytes().to_vec();
                        unlock_blocks.push(UnlockBlock::Signature(SignatureUnlock::Ed25519(Ed25519Signature {
                            public_key,
                            signature,
                        })));
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

        let inputs: Vec<Input> = inputs.into_iter().map(|(i, _)| i).collect();

        Ok(SignedTransaction {
            unsigned_transaction: UnsignedTransaction {
                inputs,
                outputs,
                payload: self.payload,
            },
            unlock_block_count: unlock_blocks.len() as u8,
            unlock_blocks,
        })
    }
}

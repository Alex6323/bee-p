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

use crate::Error;

use constants::INPUT_OUTPUT_COUNT_RANGE;

pub use essence::{TransactionEssence, TransactionEssenceBuilder};
pub use input::{Input, UTXOInput};
pub use output::{
    Address, Ed25519Address, Output, OutputId, SignatureLockedSingleOutput, WotsAddress, OUTPUT_ID_LENGTH,
};
pub use transaction_id::{TransactionId, TRANSACTION_ID_LENGTH};
pub use unlock::{Ed25519Signature, ReferenceUnlock, SignatureUnlock, UnlockBlock, WotsSignature};

use bee_common::packable::{Packable, Read, Write};

use blake2::{Blake2b, Digest};
use serde::{Deserialize, Serialize};

use alloc::{boxed::Box, vec::Vec};
use core::{cmp::Ordering, convert::TryInto, slice::Iter};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Transaction {
    essence: TransactionEssence,
    unlock_blocks: Box<[UnlockBlock]>,
}

impl Transaction {
    pub fn builder() -> TransactionBuilder {
        TransactionBuilder::default()
    }

    pub fn id(&self) -> TransactionId {
        let mut bytes = Vec::with_capacity(self.packed_len());
        let mut hasher = Blake2b::new();

        // Packing to bytes can't fail.
        self.pack(&mut bytes).unwrap();
        hasher.update(&bytes);

        // We know for sure the bytes have the right size.
        TransactionId::new(hasher.finalize()[0..TRANSACTION_ID_LENGTH].try_into().unwrap())
    }

    pub fn essence(&self) -> &TransactionEssence {
        &self.essence
    }

    pub fn unlock_blocks(&self) -> &[UnlockBlock] {
        &self.unlock_blocks
    }

    pub fn unlock_block(&self, index: usize) -> &UnlockBlock {
        let unlock_block = &self.unlock_blocks[index];
        if let UnlockBlock::Reference(reference) = unlock_block {
            &self.unlock_blocks[reference.index() as usize]
        } else {
            unlock_block
        }
    }
}

impl Packable for Transaction {
    type Error = Error;

    fn packed_len(&self) -> usize {
        self.essence.packed_len()
            + 0u16.packed_len()
            + self.unlock_blocks.iter().map(|block| block.packed_len()).sum::<usize>()
    }

    fn pack<W: Write>(&self, writer: &mut W) -> Result<(), Self::Error> {
        self.essence.pack(writer)?;

        (self.unlock_blocks.len() as u16).pack(writer)?;
        for unlock_block in self.unlock_blocks.as_ref() {
            unlock_block.pack(writer)?;
        }

        Ok(())
    }

    fn unpack<R: Read + ?Sized>(reader: &mut R) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        let essence = TransactionEssence::unpack(reader)?;

        let unlock_blocks_len = u16::unpack(reader)? as usize;
        let mut unlock_blocks = Vec::with_capacity(unlock_blocks_len);
        for _ in 0..unlock_blocks_len {
            unlock_blocks.push(UnlockBlock::unpack(reader)?);
        }

        Ok(Self {
            essence,
            unlock_blocks: unlock_blocks.into_boxed_slice(),
        })
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

#[derive(Debug, Default)]
pub struct TransactionBuilder {
    essence: Option<TransactionEssence>,
    unlock_blocks: Vec<UnlockBlock>,
}

impl TransactionBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_essence(mut self, essence: TransactionEssence) -> Self {
        self.essence = Some(essence);

        self
    }

    pub fn add_unlock_block(mut self, unlock_block: UnlockBlock) -> Self {
        self.unlock_blocks.push(unlock_block);

        self
    }

    pub fn finish(self) -> Result<Transaction, Error> {
        // TODO
        // inputs.sort();
        // outputs.sort();

        let essence = self.essence.ok_or(Error::MissingField("essence"))?;

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
                        match &essence.inputs()[i] {
                            Input::UTXO(u) => match &essence.inputs()[i - 1] {
                                Input::UTXO(v) => {
                                    if u.output_id().transaction_id() != v.output_id().transaction_id() {
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
                UnlockBlock::Signature(_) => {
                    // A Signature Unlock Block unlocking multiple inputs must only appear once (be unique) and be
                    // positioned at same index of the first input it unlocks.
                    if self.unlock_blocks.iter().filter(|j| *j == block).count() > 1 {
                        return Err(Error::DuplicateError);
                    }

                    // Since it's first input it unlocks, it must have differente transaction id from previous one
                    if i != 0 {
                        match &essence.inputs()[i] {
                            Input::UTXO(u) => match &essence.inputs()[i - 1] {
                                Input::UTXO(v) => {
                                    if u.output_id().transaction_id() == v.output_id().transaction_id() {
                                        return Err(Error::InvalidIndex);
                                    }
                                }
                            },
                        }
                    }
                }
            }
        }

        Ok(Transaction {
            essence,
            unlock_blocks: self.unlock_blocks.into_boxed_slice(),
        })
    }
}

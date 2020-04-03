use crate::transaction::{
    Hash,
    Transaction,
};

#[derive(Default)]
pub struct Transactions(pub(crate) Vec<Transaction>);

impl Transactions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get(&self, index: usize) -> Option<&Transaction> {
        self.0.get(index)
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn push(&mut self, transaction: Transaction) {
        self.0.push(transaction);
    }
}

pub struct Bundle(pub(crate) Transactions);

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

    // TODO TEST
    pub fn tail(&self) -> &Transaction {
        // Safe to unwrap because empty bundles can't be built
        self.get(0).unwrap()
    }

    // TODO TEST
    pub fn head(&self) -> &Transaction {
        // Safe to unwrap because empty bundles can't be built
        self.get(self.len() - 1).unwrap()
    }

    // TODO TEST
    pub fn trunk(&self) -> &Hash {
        self.head().trunk()
    }

    // TODO TEST
    pub fn branch(&self) -> &Hash {
        self.head().branch()
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

#[cfg(test)]
mod tests {}

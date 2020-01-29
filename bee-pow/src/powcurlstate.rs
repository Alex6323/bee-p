use crate::constants::STATE_LEN;

type WithCarry = bool;

pub(crate) struct PowCurlState {
    hi: [u64; STATE_LEN],
    lo: [u64; STATE_LEN],
}

impl PowCurlState {
    pub fn new(init_value: u64) -> Self {
        Self {
            hi: [init_value; STATE_LEN],
            lo: [init_value; STATE_LEN],
        }
    }

    pub fn set(&mut self, index: usize, hi: u64, lo: u64) {
        self.hi[index] = hi;
        self.lo[index] = lo;
    }

    pub fn get(&self, index: usize) -> (u64, u64) {
        (self.hi[index], self.lo[index])
    }

    pub fn bit_add(&mut self, index: usize) -> WithCarry {
        let hi = self.hi[index];
        let lo = self.lo[index];

        self.hi[index] = lo;
        self.lo[index] = hi ^ lo;

        (hi & !lo) != 0
    }

    pub fn bit_equal(&self, index: usize) -> u64 {
        !(self.hi[index] ^ self.lo[index])
    }

    pub unsafe fn as_mut_ptr(&mut self) -> (*mut u64, *mut u64) {
        ((&mut self.hi).as_mut_ptr(), (&mut self.lo).as_mut_ptr())
    }
}

impl Clone for PowCurlState {
    fn clone(&self) -> Self {
        let mut hi = [0; STATE_LEN];
        let mut lo = [0; STATE_LEN];

        hi.copy_from_slice(&self.hi);
        lo.copy_from_slice(&self.lo);

        Self { hi, lo }
    }
}

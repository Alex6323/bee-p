use std::ops::Range;
use crate::{Btrit, Utrit, RawEncoding, RawEncodingBuf, ShiftTernary};

const TPB: usize = 4;
const BAL: i8 = 40;

#[repr(transparent)]
pub struct T4B1([()]);

impl T4B1 {
    unsafe fn make(ptr: *const i8, offset: usize, len: usize) -> *const Self {
        let len = (len << 2) | (offset % TPB);
        std::mem::transmute((ptr.offset((offset / TPB) as isize), len))
    }

    unsafe fn ptr(&self, index: usize) -> *const i8 {
        let byte_offset = (self.len_offset().1 + index) / TPB;
        (self.0.as_ptr() as *const i8).offset(byte_offset as isize)
    }

    fn len_offset(&self) -> (usize, usize) {
        (self.0.len() >> 2, self.0.len() & 0b11)
    }
}

fn extract(x: i8, elem: usize) -> Btrit {
    if elem < TPB {
        Utrit::from_u8((((x + BAL) / 3i8.pow(elem as u32)) % 3) as u8).shift()
    } else {
        unreachable!("Attempted to extract invalid element {} from balanced T4B1", elem)
    }
}

fn insert(x: i8, elem: usize, trit: Btrit) -> i8 {
    if elem < TPB {
        let utrit = trit.shift();
        let ux = x + BAL;
        let ux = ux + (utrit.into_u8() as i8 - (ux / 3i8.pow(elem as u32)) % 3) * 3i8.pow(elem as u32);
        ux - BAL
    } else {
        unreachable!("Attempted to insert invalid element {} into balanced T4B1", elem)
    }
}

impl RawEncoding for T4B1 {
    type Trit = Btrit;
    type Buf = T4B1Buf;

    fn empty() -> &'static Self {
        unsafe { &*Self::make(&[] as *const _, 0, 0) }
    }

    fn len(&self) -> usize {
        self.len_offset().0
    }

    fn as_i8_slice(&self) -> &[i8] {
        unsafe { &*(Self::make(self.ptr(0), 0, (self.len() as f32 / TPB as f32).ceil() as usize) as *const _) }
    }

    unsafe fn as_i8_slice_mut(&mut self) -> &mut [i8] {
        &mut *(Self::make(self.ptr(0), 0, (self.len() as f32 / TPB as f32).ceil() as usize) as *mut _)
    }

    unsafe fn get_unchecked(&self, index: usize) -> Self::Trit {
        let b = self.ptr(index).read();
        extract(b, (self.len_offset().1 + index) % TPB)
    }

    unsafe fn set_unchecked(&mut self, index: usize, trit: Self::Trit) {
        let b = self.ptr(index).read();
        let b = insert(b, (self.len_offset().1 + index) % TPB, trit);
        (self.ptr(index) as *mut i8).write(b);
    }

    unsafe fn slice_unchecked(&self, range: Range<usize>) -> &Self {
        &*Self::make(self.ptr(range.start), (self.len_offset().1 + range.start) % TPB, range.end - range.start)
    }

    unsafe fn slice_unchecked_mut(&mut self, range: Range<usize>) -> &mut Self {
        &mut *(Self::make(self.ptr(range.start), (self.len_offset().1 + range.start) % TPB, range.end - range.start) as *mut Self)
    }

    fn is_valid(b: &i8) -> bool { *b >= -BAL && *b <= BAL }

    unsafe fn from_raw_unchecked(b: &[i8], num_trits: usize) -> &Self {
        debug_assert!(num_trits <= b.len()*TPB);
        &*Self::make(b.as_ptr() as *const _, 0, num_trits)
    }

    unsafe fn from_raw_unchecked_mut(b: &mut [i8], num_trits: usize) -> &mut Self {
        debug_assert!(num_trits <= b.len()*TPB);
        &mut *(Self::make(b.as_ptr() as *const _, 0, num_trits) as *mut _)
    }
}

#[derive(Clone)]
pub struct T4B1Buf(Vec<i8>, usize);

impl RawEncodingBuf for T4B1Buf {
    type Slice = T4B1;

    fn new() -> Self {
        Self(Vec::new(), 0)
    }

    fn push(&mut self, trit: <Self::Slice as RawEncoding>::Trit) {
        if self.1 % TPB == 0 {
            self.0.push(insert(0, 0, trit));
        } else {
            let last_index = self.0.len() - 1;
            let b = unsafe { self.0.get_unchecked_mut(last_index) };
            *b = insert(*b, self.1 % TPB, trit);
        }
        self.1 += 1;
    }

    fn pop(&mut self) -> Option<<Self::Slice as RawEncoding>::Trit> {
        let val = if self.1 == 0 {
            return None;
        } else if self.1 % TPB == 1 {
            self.0.pop().map(|b| extract(b, 0))
        } else {
            let last_index = self.0.len() - 1;
            unsafe { Some(extract(*self.0.get_unchecked(last_index), (self.1 + TPB - 1) % TPB)) }
        };
        self.1 -= 1;
        val
    }

    fn as_slice(&self) -> &Self::Slice {
        unsafe { &*Self::Slice::make(self.0.as_ptr() as _, 0, self.1) }
    }

    fn as_slice_mut(&mut self) -> &mut Self::Slice {
        unsafe { &mut *(Self::Slice::make(self.0.as_ptr() as _, 0, self.1) as *mut _) }
    }
}

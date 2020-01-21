use std::ops::Range;
use crate::{Trit, RawEncoding, RawEncodingBuf};

#[repr(transparent)]
pub struct T4B1([()]);

impl T4B1 {
    unsafe fn make(ptr: *const u8, offset: usize, len: usize) -> *const Self {
        let len = (len << 2) | (offset % 4);
        std::mem::transmute((ptr.offset((offset / 4) as isize), len))
    }

    unsafe fn ptr(&self, index: usize) -> *const u8 {
        let byte_offset = index / 4;
        (self.0.as_ptr() as *const u8).offset(byte_offset as isize)
    }

    fn len_offset(&self) -> (usize, usize) {
        (self.0.len() >> 2, self.0.len() & 0b11)
    }
}

impl RawEncoding for T4B1 {
    fn len(&self) -> usize {
        self.len_offset().0
    }

    unsafe fn get_unchecked(&self, index: usize) -> Trit {
        let b = self.ptr(index).read();
        let trit = (b >> (((self.len_offset().1 + index) % 4) * 2)) & 0b11;
        Trit::from_u8(trit)
    }

    unsafe fn set_unchecked(&mut self, index: usize, trit: Trit) {
        let b = self.ptr(index).read();
        let b = b & !(0b11 << (((self.len_offset().1 + index) % 4) * 2));
        let b = b | (trit.into_u8() << (self.len_offset().1 * 2));
        (self.ptr(index) as *mut u8).write(b);
    }

    unsafe fn slice_unchecked(&self, range: Range<usize>) -> &Self {
        &*Self::make(self.ptr(0), range.start, range.end - range.start)
    }

    unsafe fn slice_unchecked_mut(&mut self, range: Range<usize>) -> &mut Self {
        &mut *(Self::make(self.ptr(0), range.start, range.end - range.start) as *mut Self)
    }
}

pub struct T4B1Buf(Vec<u8>, usize);

impl RawEncodingBuf for T4B1Buf {
    type Slice = T4B1;

    fn new() -> Self {
        Self(Vec::new(), 0)
    }

    fn push(&mut self, trit: Trit) {
        let b = trit.into_u8();
        if self.1 % 4 == 0 {
            self.0.push(b);
        } else {
            let last_index = self.0.len() - 1;
            unsafe { *self.0.get_unchecked_mut(last_index) |= b << ((self.1 % 4) * 2) };
        }
        self.1 += 1;
    }

    fn as_slice(&self) -> &Self::Slice {
        unsafe { &*Self::Slice::make(self.0.as_ptr() as _, 0, self.1) }
    }

    fn as_slice_mut(&mut self) -> &mut Self::Slice {
        unsafe { &mut *(Self::Slice::make(self.0.as_ptr() as _, 0, self.1) as *mut _) }
    }
}

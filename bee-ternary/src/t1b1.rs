use std::ops::Range;
use crate::{
    Trit,
    RawEncoding,
    RawEncodingBuf,
};

#[repr(transparent)]
pub struct T1B1([()]);

impl T1B1 {
    unsafe fn make(ptr: *const u8, offset: usize, len: usize) -> *const Self {
        std::mem::transmute((ptr.offset(offset as isize), len))
    }

    unsafe fn ptr(&self, index: usize) -> *const u8 {
        (self.0.as_ptr() as *const u8).offset(index as isize)
    }
}

impl RawEncoding for T1B1 {
    fn len(&self) -> usize {
        self.0.len()
    }

    unsafe fn get_unchecked(&self, index: usize) -> Trit {
        Trit::from_u8(self.ptr(index).read())
    }

    unsafe fn set_unchecked(&mut self, index: usize, trit: Trit) {
        (self.ptr(index) as *mut u8).write(trit.into_u8());
    }

    unsafe fn slice_unchecked(&self, range: Range<usize>) -> &Self {
        &*Self::make(self.ptr(0), range.start, range.end - range.start)
    }

    unsafe fn slice_unchecked_mut(&mut self, range: Range<usize>) -> &mut Self {
        &mut *(Self::make(self.ptr(0), range.start, range.end - range.start) as *mut _)
    }
}

pub struct T1B1Buf(Vec<u8>);

impl RawEncodingBuf for T1B1Buf {
    type Slice = T1B1;

    fn new() -> Self {
        Self(Vec::new())
    }

    fn push(&mut self, trit: Trit) {
        self.0.push(trit.into_u8());
    }

    fn as_slice(&self) -> &Self::Slice {
        unsafe { &*Self::Slice::make(self.0.as_ptr() as _, 0, self.0.len()) }
    }

    fn as_slice_mut(&mut self) -> &mut Self::Slice {
        unsafe { &mut *(Self::Slice::make(self.0.as_ptr() as _, 0, self.0.len()) as *mut _) }
    }
}

use std::ops::Range;
use crate::{
    Trit,
    RawEncoding,
    RawEncodingBuf,
};

#[repr(transparent)]
pub struct T1B1([()]);

impl T1B1 {
    unsafe fn make(ptr: *const i8, offset: usize, len: usize) -> *const Self {
        std::mem::transmute((ptr.offset(offset as isize), len))
    }

    unsafe fn ptr(&self, index: usize) -> *const i8 {
        (self.0.as_ptr() as *const i8).offset(index as isize)
    }

    pub fn as_i8_slice(&self) -> &[i8] {
        unsafe { &*(Self::make(self.ptr(0), 0, self.len()) as *const _) }
    }

    pub unsafe fn as_i8_slice_mut(&mut self) -> &mut [i8] {
        &mut *(Self::make(self.ptr(0), 0, self.len()) as *mut _)
    }
}

impl RawEncoding for T1B1 {
    fn empty() -> &'static Self {
        unsafe { &*Self::make(&[] as *const _, 0, 0) }
    }

    fn len(&self) -> usize {
        self.0.len()
    }

    unsafe fn get_unchecked(&self, index: usize) -> Trit {
        Trit::from(self.ptr(index).read())
    }

    unsafe fn set_unchecked(&mut self, index: usize, trit: Trit) {
        (self.ptr(index) as *mut i8).write(trit.into());
    }

    unsafe fn slice_unchecked(&self, range: Range<usize>) -> &Self {
        &*Self::make(self.ptr(0), range.start, range.end - range.start)
    }

    unsafe fn slice_unchecked_mut(&mut self, range: Range<usize>) -> &mut Self {
        &mut *(Self::make(self.ptr(0), range.start, range.end - range.start) as *mut _)
    }
}

#[derive(Clone)]
pub struct T1B1Buf(Vec<i8>);

impl RawEncodingBuf for T1B1Buf {
    type Slice = T1B1;

    fn new() -> Self {
        Self(Vec::new())
    }

    fn push(&mut self, trit: Trit) {
        self.0.push(trit.into());
    }

    fn as_slice(&self) -> &Self::Slice {
        unsafe { &*Self::Slice::make(self.0.as_ptr() as _, 0, self.0.len()) }
    }

    fn as_slice_mut(&mut self) -> &mut Self::Slice {
        unsafe { &mut *(Self::Slice::make(self.0.as_ptr() as _, 0, self.0.len()) as *mut _) }
    }
}

use std::{
    marker::PhantomData,
    ops::Range,
};
use crate::{
    Trit, Utrit, Btrit,
    RawEncoding,
    RawEncodingBuf,
};

#[repr(transparent)]
pub struct T1B1<T: Trit = Btrit> {
    _phantom: PhantomData<T>,
    inner: [()],
}

impl<T: Trit> T1B1<T> {
    unsafe fn make(ptr: *const T, offset: usize, len: usize) -> *const Self {
        std::mem::transmute((ptr.offset(offset as isize), len))
    }

    unsafe fn ptr(&self, index: usize) -> *const T {
        (self.inner.as_ptr() as *const T).offset(index as isize)
    }
}

impl T1B1<Btrit> {
    pub fn as_i8_slice(&self) -> &[i8] {
        unsafe { &*(Self::make(self.ptr(0), 0, self.len()) as *const _) }
    }

    pub unsafe fn as_i8_slice_mut(&mut self) -> &mut [i8] {
        &mut *(Self::make(self.ptr(0), 0, self.len()) as *mut _)
    }
}

impl<T: Trit> T1B1<T> {
    pub fn as_raw_slice(&self) -> &[T] {
        unsafe { &*(Self::make(self.ptr(0), 0, self.len()) as *const _) }
    }

    pub fn as_raw_slice_mut(&mut self) -> &mut [T] {
        unsafe { &mut *(Self::make(self.ptr(0), 0, self.len()) as *mut _) }
    }
}

impl<T: Trit> RawEncoding for T1B1<T> {
    fn empty() -> &'static Self {
        unsafe { &*Self::make(&[] as *const _, 0, 0) }
    }

    fn len(&self) -> usize {
        self.inner.len()
    }

    unsafe fn get_unchecked(&self, index: usize) -> Utrit {
        self.ptr(index).read().into()
    }

    unsafe fn set_unchecked(&mut self, index: usize, trit: Utrit) {
        (self.ptr(index) as *mut T).write(trit.into());
    }

    unsafe fn slice_unchecked(&self, range: Range<usize>) -> &Self {
        &*Self::make(self.ptr(0), range.start, range.end - range.start)
    }

    unsafe fn slice_unchecked_mut(&mut self, range: Range<usize>) -> &mut Self {
        &mut *(Self::make(self.ptr(0), range.start, range.end - range.start) as *mut _)
    }
}

#[derive(Clone)]
pub struct T1B1Buf<T: Trit = Btrit> {
    _phantom: PhantomData<T>,
    inner: Vec<T>,
}

impl<T: Trit> RawEncodingBuf for T1B1Buf<T> {
    type Slice = T1B1<T>;

    fn new() -> Self {
        Self {
            _phantom: PhantomData,
            inner: Vec::new(),
        }
    }

    fn push(&mut self, trit: Utrit) {
        self.inner.push(trit.into());
    }

    fn pop(&mut self) -> Option<Utrit> {
        self.inner.pop().map(Into::into)
    }

    fn as_slice(&self) -> &Self::Slice {
        unsafe { &*Self::Slice::make(self.inner.as_ptr() as _, 0, self.inner.len()) }
    }

    fn as_slice_mut(&mut self) -> &mut Self::Slice {
        unsafe { &mut *(Self::Slice::make(self.inner.as_ptr() as _, 0, self.inner.len()) as *mut _) }
    }
}

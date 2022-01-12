/// The crate arrayvec doesn't currently have a Copy version of ArrayVec, this implements it
use std::{
    mem::MaybeUninit,
    ops::{Deref, DerefMut},
};

#[derive(Copy)]
pub struct ArrayVecCopy<T, const N: usize> {
    data: [MaybeUninit<T>; N],
    len: usize,
}
impl<T: Copy, const N: usize> ArrayVecCopy<T, N> {
    pub fn new() -> Self {
        Self {
            data: [MaybeUninit::uninit(); N],
            len: 0,
        }
    }

    #[inline(always)]
    pub fn len(&self) -> usize {
        self.len
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn push(&mut self, data: T) {
        if self.len < N {
            self.data[self.len].write(data);
            self.len += 1;
        }
    }

    pub fn as_slice(&self) -> &[T] {
        unsafe { std::slice::from_raw_parts(self.as_ptr(), self.len()) }
    }
    pub fn as_mut_slice(&mut self) -> &mut [T] {
        unsafe { std::slice::from_raw_parts_mut(self.as_mut_ptr(), self.len()) }
    }

    #[inline]
    pub fn as_ptr(&self) -> *const T {
        self.data.as_ptr() as _
    }
    #[inline]
    pub fn as_mut_ptr(&mut self) -> *mut T {
        self.data.as_mut_ptr() as _
    }

    #[inline]
    unsafe fn get_ptr(&self, ix: usize) -> *const T {
        debug_assert!(ix < N);
        unsafe { self.data[ix].as_ptr() }
    }

    #[inline]
    pub fn iter(&self) -> Iter<T, N> {
        Iter {
            parent: self,
            ix: 0,
        }
    }
}

impl<T: Copy, const N: usize> Clone for ArrayVecCopy<T, N> {
    fn clone(&self) -> Self {
        Self {
            data: self.data.clone(),
            len: self.len.clone(),
        }
    }
}

impl<T: Copy, const N: usize> Default for ArrayVecCopy<T, N> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Copy, const N: usize> Deref for ArrayVecCopy<T, N> {
    type Target = [T];

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}
impl<T: Copy, const N: usize> DerefMut for ArrayVecCopy<T, N> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_mut_slice()
    }
}
impl<'a, T: Copy + 'a, const N: usize> IntoIterator for &'a ArrayVecCopy<T, N> {
    type Item = &'a T;

    type IntoIter = Iter<'a, T, N>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<T: Copy + std::fmt::Debug, const N: usize> std::fmt::Debug for ArrayVecCopy<T, N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(&self[0..self.len], f)
    }
}
pub struct Iter<'a, T, const N: usize> {
    parent: &'a ArrayVecCopy<T, N>,
    ix: usize,
}
impl<'a, T: Copy, const N: usize> Iterator for Iter<'a, T, N> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.ix < self.parent.len() {
            let ret = unsafe { Some(&*self.parent.get_ptr(self.ix)) };

            self.ix += 1;

            ret
        } else {
            None
        }
    }
}

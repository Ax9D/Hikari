use std::any::{type_name, Any};
use std::cell::{Cell, UnsafeCell};
use std::fmt;
use std::ops::{Deref, DerefMut};

use crate::State;

pub struct Ref<'a, T> {
    data: &'a T,
    borrow: &'a Cell<isize>,
}

impl<'a, T> Drop for Ref<'a, T> {
    fn drop(&mut self) {
        self.borrow.set(self.borrow.get() + 1);
    }
}
impl<'a, T> Deref for Ref<'a, T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.data
    }
}
impl<T> AsRef<T> for Ref<'_, T> {
    fn as_ref(&self) -> &T {
        &*self
    }
}
use std::borrow::Borrow;
impl<T> Borrow<T> for Ref<'_, T> {
    fn borrow(&self) -> &T {
        &*self
    }
}

impl<'a, S: fmt::Debug> fmt::Debug for Ref<'a, S> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.data.fmt(f)
    }
}
impl<T: fmt::Display> fmt::Display for Ref<'_, T> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.data.fmt(f)
    }
}

pub struct RefMut<'a, T> {
    data: &'a mut T,
    borrow: &'a Cell<isize>,
}

impl<T> Drop for RefMut<'_, T> {
    #[inline]
    fn drop(&mut self) {
        self.borrow.set(self.borrow.get() - 1);
    }
}
impl<T> Deref for RefMut<'_, T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.data
    }
}
impl<T> DerefMut for RefMut<'_, T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.data
    }
}

impl<T> AsRef<T> for RefMut<'_, T> {
    fn as_ref(&self) -> &T {
        &*self
    }
}
impl<T> AsMut<T> for RefMut<'_, T> {
    fn as_mut(&mut self) -> &mut T {
        &mut *self
    }
}
impl<T> Borrow<T> for RefMut<'_, T> {
    fn borrow(&self) -> &T {
        &*self
    }
}

impl<'a, S: fmt::Debug> fmt::Debug for RefMut<'a, S> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.data.fmt(f)
    }
}
impl<T: fmt::Display> fmt::Display for RefMut<'_, T> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.data.fmt(f)
    }
}

pub struct StateCell {
    data: UnsafeCell<Box<dyn Any>>,
    borrow: Cell<isize>,
}

impl StateCell {
    pub fn new<S: State>(state: S) -> Self {
        Self {
            data: UnsafeCell::new(Box::new(state)),
            borrow: Cell::new(0),
        }
    }
    pub fn borrow_cast<S: State>(&self) -> Ref<S> {
        //Negative is shared reference count
        if self.borrow.get() <= 0 {
            let data_ref = unsafe { &*self.data.get() };
            let typed_ref = data_ref.downcast_ref::<S>().unwrap();

            self.borrow.set(self.borrow.get() - 1);

            Ref {
                data: typed_ref,
                borrow: &self.borrow,
            }
        } else {
            panic!(
                "Couldn't get a shared reference to {} as it is already borrowed mutably",
                type_name::<S>()
            );
        }
    }
    pub fn borrow_cast_mut<S: State>(&self) -> RefMut<S> {
        //Negative is shared reference count
        if self.borrow.get() == 0 {
            let data_ref = unsafe { &mut *self.data.get() };
            let typed_ref = data_ref.downcast_mut::<S>().unwrap();

            self.borrow.set(self.borrow.get() + 1);

            RefMut {
                data: typed_ref,
                borrow: &self.borrow,
            }
        } else {
            panic!(
                "Couldn't get an exculsive reference to {} as it is already borrowed exclusively",
                type_name::<S>()
            );
        }
    }
}

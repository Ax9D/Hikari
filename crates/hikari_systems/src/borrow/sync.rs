use std::{
    any::Any,
    cell::UnsafeCell,
    fmt,
    ops::{Deref, DerefMut},
    sync::atomic::{self, AtomicUsize},
};

use crate::State;

/* Atomic borrow counting code has been stolen from Atomic RefCell's implementation,
 * by bholley licensed under the Mozilla Public License, http://mozilla.org/MPL/2.0
 */
pub struct StateCell {
    data: UnsafeCell<Box<dyn Any>>,
    borrow: AtomicUsize,
}

impl StateCell {
    pub fn new<S: State>(state: S) -> Self {
        Self {
            data: UnsafeCell::new(Box::new(state)),
            borrow: AtomicUsize::new(0),
        }
    }
    pub fn borrow_cast<S: State>(&self) -> Ref<S> {
        match BorrowRef::try_new(&self.borrow) {
            Ok(borrow) => {
                let data_ref = unsafe { &*self.data.get() };
                let typed_ref = data_ref.downcast_ref::<S>().unwrap();

                Ref {
                    data: typed_ref,
                    borrow,
                }
            }
            Err(s) => panic!("{}", s),
        }
    }
    pub fn borrow_cast_mut<S: State>(&self) -> RefMut<S> {
        match BorrowRefMut::try_new(&self.borrow) {
            Ok(borrow) => {
                let data_ref = unsafe { &mut *self.data.get() };
                let typed_ref = data_ref.downcast_mut::<S>().unwrap();

                RefMut {
                    data: typed_ref,
                    borrow,
                }
            }
            Err(s) => panic!("{}", s),
        }
    }

    pub unsafe fn borrow_cast_unchecked<S: State>(&self) -> &S {
        let data_ref = &mut *self.data.get();
        let typed_ref = data_ref.downcast_ref::<S>().unwrap();

        typed_ref
    }
    pub unsafe fn borrow_cast_unchecked_mut<S: State>(&self) -> &mut S {
        let data_ref = &mut *self.data.get();
        let typed_ref = data_ref.downcast_mut::<S>().unwrap();

        typed_ref
    }
}
#[derive(Debug)]
struct BorrowRef<'a> {
    borrow: &'a AtomicUsize,
}

const HIGH_BIT: usize = !(::core::usize::MAX >> 1);
const MAX_FAILED_BORROWS: usize = HIGH_BIT + (HIGH_BIT >> 1);

impl<'b> BorrowRef<'b> {
    #[inline]
    fn try_new(borrow: &'b AtomicUsize) -> Result<Self, &'static str> {
        let new = borrow.fetch_add(1, atomic::Ordering::Acquire) + 1;
        if new & HIGH_BIT != 0 {
            // If the new count has the high bit set, that almost certainly
            // means there's an pre-existing mutable borrow. In that case,
            // we simply leave the increment as a benign side-effect and
            // return `Err`. Once the mutable borrow is released, the
            // count will be reset to zero unconditionally.
            //
            // The overflow check here ensures that an unbounded number of
            // immutable borrows during the scope of one mutable borrow
            // will soundly trigger a panic (or abort) rather than UB.
            Self::check_overflow(borrow, new);
            Err("already mutably borrowed")
        } else {
            Ok(BorrowRef { borrow })
        }
    }

    #[cold]
    #[inline(never)]
    fn check_overflow(borrow: &'b AtomicUsize, new: usize) {
        if new == HIGH_BIT {
            // We overflowed into the reserved upper half of the refcount
            // space. Before panicking, decrement the refcount to leave things
            // in a consistent immutable-borrow state.
            //
            // This can basically only happen if somebody forget()s AtomicRefs
            // in a tight loop.
            borrow.fetch_sub(1, atomic::Ordering::Release);
            panic!("too many immutable borrows");
        } else if new >= MAX_FAILED_BORROWS {
            // During the mutable borrow, an absurd number of threads have
            // attempted to increment the refcount with immutable borrows.
            // To avoid hypothetically wrapping the refcount, we abort the
            // process once a certain threshold is reached.
            //
            // This requires billions of borrows to fail during the scope of
            // one mutable borrow, and so is very unlikely to happen in a real
            // program.
            //
            // To avoid a potential unsound state after overflowing, we make
            // sure the entire process aborts.
            //
            // Right now, there's no stable way to do that without `std`:
            // https://github.com/rust-lang/rust/issues/67952
            // As a workaround, we cause an abort by making this thread panic
            // during the unwinding of another panic.
            //
            // On platforms where the panic strategy is already 'abort', the
            // ForceAbort object here has no effect, as the program already
            // panics before it is dropped.
            struct ForceAbort;
            impl Drop for ForceAbort {
                fn drop(&mut self) {
                    panic!("Aborting to avoid unsound state of AtomicRefCell");
                }
            }
            let _abort = ForceAbort;
            panic!("Too many failed borrows");
        }
    }
}
impl<'b> Drop for BorrowRef<'b> {
    #[inline]
    fn drop(&mut self) {
        let old = self.borrow.fetch_sub(1, atomic::Ordering::Release);
        // This assertion is technically incorrect in the case where another
        // thread hits the hypothetical overflow case, since we might observe
        // the refcount before it fixes it up (and panics). But that never will
        // never happen in a real program, and this is a debug_assert! anyway.
        debug_assert!(old & HIGH_BIT == 0);
    }
}
#[derive(Debug)]
struct BorrowRefMut<'b> {
    borrow: &'b AtomicUsize,
}

impl<'b> Drop for BorrowRefMut<'b> {
    #[inline]
    fn drop(&mut self) {
        self.borrow.store(0, atomic::Ordering::Release);
    }
}

impl<'b> BorrowRefMut<'b> {
    #[inline]
    fn try_new(borrow: &'b AtomicUsize) -> Result<BorrowRefMut<'b>, &'static str> {
        // Use compare-and-swap to avoid corrupting the immutable borrow count
        // on illegal mutable borrows.
        let old = match borrow.compare_exchange(
            0,
            HIGH_BIT,
            atomic::Ordering::Acquire,
            atomic::Ordering::Relaxed,
        ) {
            Ok(x) => x,
            Err(x) => x,
        };

        if old == 0 {
            Ok(BorrowRefMut { borrow })
        } else if old & HIGH_BIT == 0 {
            Err("already immutably borrowed")
        } else {
            Err("already mutably borrowed")
        }
    }
}
#[allow(dead_code)]
pub struct Ref<'a, S: 'a> {
    data: &'a S,
    borrow: BorrowRef<'a>,
}

impl<T> Deref for Ref<'_, T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &T {
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

#[allow(dead_code)]
pub struct RefMut<'a, S: 'a> {
    data: &'a mut S,
    borrow: BorrowRefMut<'a>,
}

impl<T> Deref for RefMut<'_, T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &T {
        self.data
    }
}

impl<T> DerefMut for RefMut<'_, T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut T {
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
use std::borrow::BorrowMut;
impl<T> BorrowMut<T> for RefMut<'_, T> {
    fn borrow_mut(&mut self) -> &mut T {
        &mut *self
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

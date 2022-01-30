use std::{
    cell::{Ref, RefCell, RefMut},
    sync::Arc,
};

use atomic_refcell::AtomicRefCell;

//Aware of the sideffects
// pub struct UnsafeShared<T> {
//     inner: Rc<UnsafeCell<T>>,
// }
// impl<T> UnsafeShared<T> {
//     pub fn new(data: T) -> Self {
//         Self {
//             inner: Rc::new(UnsafeCell::new(data)),
//         }
//     }
//     #[inline(always)]
//     pub fn borrow(&self) -> &T {
//         unsafe { &*(self.inner.as_ref().get() as *const T) }
//     }
//     #[inline(always)]
//     pub fn borrow_mut(&self) -> &mut T {
//         unsafe { &mut *self.inner.as_ref().get() }
//     }
// }
// impl<T> Clone for UnsafeShared<T> {
//     fn clone(&self) -> Self {
//         Self {
//             inner: self.inner.clone(),
//         }
//     }
// }

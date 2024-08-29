use std::{borrow::Borrow, ptr::NonNull, sync::atomic::{fence, AtomicUsize, Ordering}};
use std::hash::Hash;

use crate::common::*;

struct SharedCounter {
    index: usize,
    sender: flume::Sender<RefMessage>,
    strong: AtomicUsize,
    weak: AtomicUsize,
    internal: AtomicUsize,
}

unsafe impl Send for RawHandle {}
unsafe impl Sync for RawHandle {}

pub struct RawHandle {
    ref_type: RefType,
    inner: NonNull<SharedCounter>
}
impl RawHandle {
    pub fn strong(index: usize, sender: flume::Sender<RefMessage>) -> Self {
        let inner = Box::new(SharedCounter {
            index,
            sender,
            strong: AtomicUsize::new(1),
            weak: AtomicUsize::new(0),
            internal: AtomicUsize::new(0)
        });

        let inner = Box::leak(inner).into();
        Self {
            ref_type: RefType::Strong,
            inner
        }
    }
    fn inner(&self) -> &SharedCounter {
        unsafe { self.inner.as_ref() }
    }
    fn increment_strong(&self) -> usize {
        self.inner().strong.fetch_add(1, Ordering::Relaxed)
    }
    #[allow(unused)]
    fn decrement_strong(&self) -> usize {
        self.inner().strong.fetch_sub(1, Ordering::Relaxed)
    }
    fn increment_weak(&self) -> usize {
        self.inner().weak.fetch_add(1, Ordering::Relaxed)
    }
    fn increment_internal(&self) -> usize {
        self.inner().internal.fetch_add(1, Ordering::Relaxed)
    }
    pub fn index(&self) -> usize {
        self.inner().index
    }

    #[doc(hidden)]
    pub fn __upgrade_strong_anyway(&self) -> Self {
        self.upgrade_strong().unwrap_or_else(|| {
            assert!(self.weak_count() != 0 || self.is_internal());
            
            self.increment_strong();
            Self {
                ref_type: RefType::Strong,
                inner: self.inner
            }
        })
    }
    pub fn upgrade_strong(&self) -> Option<Self> {

        fn check_increment(x: usize) -> Option<usize> {
            if x == 0 {
                None
            } else {
                Some(x + 1)
            }
        } 

        match self.ref_type {
            RefType::Strong => {
                self.increment_strong();

                Some(
                    Self {
                        ref_type: RefType::Strong,
                        inner: self.inner
                    }
                )
            },
            RefType::Weak | RefType::Internal => {
                if self.inner().strong.fetch_update(Ordering::Acquire, Ordering::Relaxed, check_increment).is_ok() {
                    Some(
                        Self {
                        ref_type: RefType::Strong,
                        inner: self.inner
                        }
                    )

                } else {
                    None
                }
            },
        }
    }
    pub fn upgrade_weak(&self) -> Self {
        match self.ref_type {
            RefType::Internal => {
                self.increment_weak();

                Self {
                    ref_type: RefType::Weak,
                    inner: self.inner
                }
            }
            _=> panic!("Can only upgrade from internal to weak")
        }
    }
    pub fn downgrade_weak(&self) -> Self {

        match self.ref_type {
            RefType::Strong | RefType::Weak => {
                self.increment_weak();

                Self {
                    ref_type: RefType::Weak,
                    inner: self.inner
                }
            }
            RefType::Internal => panic!("Internal is weaker than weak, can't downgrade"),
        }
    }
    pub fn downgrade_internal(&self) -> Self {
        match self.ref_type {
            RefType::Strong | RefType::Weak | RefType::Internal => {
                self.increment_internal();

                Self {
                    ref_type: RefType::Internal,
                    inner: self.inner
                }
            }
        }
    }
    pub fn strong_count(&self) -> usize {
        self.inner().strong.load(Ordering::Relaxed)
    }
    pub fn weak_count(&self) -> usize { 
        self.inner().weak.load(Ordering::Relaxed)
    }
    pub fn is_strong(&self) -> bool {
        self.ref_type.is_strong()
    }
    pub fn is_weak(&self) -> bool {
        self.ref_type.is_weak()
    }
    pub fn is_internal(&self) -> bool {
        self.ref_type.is_internal()
    }
}
impl Clone for RawHandle {
    fn clone(&self) -> Self {
        match self.ref_type {
            RefType::Strong => {
                self.increment_strong();
            },
            RefType::Weak => {
                self.increment_weak();
            },
            RefType::Internal => {
                self.increment_internal();
            },
        }

        Self { ref_type: self.ref_type.clone(), inner: self.inner.clone() }
    }
}
impl Drop for RawHandle {
    fn drop(&mut self) {
        let inner = self.inner();
        match self.ref_type {
            RefType::Weak => {
                let old_weak_count = inner.weak.fetch_sub(1, Ordering::Release);
                let weak_count = old_weak_count - 1;
                if weak_count == 0 {
                    let strong_count = self.strong_count();

                    if strong_count == 0 {
                        inner.sender.send(RefMessage::Deallocate(inner.index)).expect("Failed to send ref message");
                    }
                }

            },
            RefType::Strong => {
                let old_strong_count = inner.strong.fetch_sub(1, Ordering::Release);
                let strong_count = old_strong_count - 1;
                if strong_count == 0 {
                    inner.sender.send(RefMessage::Unload(inner.index)).expect("Failed to send ref message");
                    
                    let weak_count = self.weak_count();
                    if weak_count == 0 {
                        inner.sender.send(RefMessage::Deallocate(inner.index)).expect("Failed to send ref message");
                    }
                }
                
            },
            RefType::Internal => {
                let old_internal_count = inner.internal.fetch_sub(1, Ordering::Release);

                if old_internal_count == 1 && self.strong_count() == 0 && self.weak_count() == 0 {
                    // Free Box allocation
                    let _dropped = unsafe { Box::from_raw(self.inner.as_ptr()) };
                }
            }
        }
        
        // Don't know quite what this does but std:sync::Arc does it
        fence(Ordering::Acquire);
    }
}

impl Borrow<usize> for RawHandle {
    fn borrow(&self) -> &usize {
        &self.inner().index
    }
}

impl PartialEq for RawHandle {
    fn eq(&self, other: &Self) -> bool {
        self.inner == other.inner
    }
}
impl Eq for RawHandle {}

impl Hash for RawHandle {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.inner.hash(state);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use flume::unbounded;
    #[test]
    fn test_strong_creation() {
        let (sender, _receiver) = unbounded();
        let handle = RawHandle::strong(1, sender);
        assert_eq!(handle.strong_count(), 1);
        assert_eq!(handle.weak_count(), 0);
    }

    #[test]
    fn test_upgrade_strong_from_weak() {
        let (sender, _receiver) = unbounded();
        let handle = RawHandle::strong(1, sender);
        let weak_handle = handle.downgrade_weak();
        let upgraded_handle = weak_handle.upgrade_strong().unwrap();
        assert_eq!(upgraded_handle.strong_count(), 2);
    }

    #[test]
    fn test_upgrade_strong_failure() {
        let (sender, _receiver) = unbounded();
        let handle = RawHandle::strong(1, sender);
        let weak_handle = handle.downgrade_weak();
        drop(handle);
        assert!(weak_handle.upgrade_strong().is_none());
    }

    #[test]
    fn test_downgrade_to_weak() {
        let (sender, _receiver) = unbounded();
        let handle = RawHandle::strong(1, sender);
        let _weak_handle = handle.downgrade_weak();
        assert_eq!(handle.weak_count(), 1);
    }

    #[test]
    fn test_clone_strong_handle() {
        let (sender, _receiver) = unbounded();
        let handle = RawHandle::strong(1, sender);
        let cloned_handle = handle.clone();
        assert_eq!(cloned_handle.strong_count(), 2);
        assert_eq!(cloned_handle.weak_count(), 0);
    }

    #[test]
    fn test_clone_weak_handle() {
        let (sender, _receiver) = unbounded();
        let handle = RawHandle::strong(1, sender);
        let weak_handle = handle.downgrade_weak();
        let cloned_weak_handle = weak_handle.clone();
        assert_eq!(cloned_weak_handle.weak_count(), 2);
        assert_eq!(cloned_weak_handle.strong_count(), 1);
    }

    #[test]
    fn test_drop_strong_handle() {
        let (sender, receiver) = unbounded();
        {
            let _handle = RawHandle::strong(1, sender.clone());
        }
        let messages: Vec<_> = receiver.drain().collect();
        assert_eq!(messages, vec![RefMessage::Unload(1), RefMessage::Deallocate(1)]);
    }

    #[test]
    fn test_drop_weak_handle() {
        let (sender, receiver) = unbounded();
        let handle = RawHandle::strong(1, sender.clone());
        let weak_handle = handle.downgrade_weak();
        let cloned_weak_handle = weak_handle.clone();
        drop(handle);

        let message = receiver.try_recv().unwrap();

        assert_eq!(message, RefMessage::Unload(1));
        drop(weak_handle);

        assert!(cloned_weak_handle.strong_count() == 0);
        assert!(cloned_weak_handle.weak_count() == 1);

        assert!(receiver.try_recv().is_err());
    }

    #[test]
    fn test_drop_last_weak_handle() {
        let (sender, receiver) = unbounded();
        let handle = RawHandle::strong(1, sender.clone());
        let _weak_handle = handle.downgrade_weak();
        drop(handle);
        
        let message = receiver.try_recv().unwrap();
        assert_eq!(message, RefMessage::Unload(1));
        assert!(receiver.try_recv().is_err());
        
        drop(_weak_handle);
        
        let message = receiver.try_recv().unwrap();
        assert_eq!(message, RefMessage::Deallocate(1));
    }

    #[test]
    fn test_downgrade_to_internal() {
        let (sender, _receiver) = unbounded();
        let handle = RawHandle::strong(1, sender);
        let internal_handle = handle.downgrade_internal();
        assert_eq!(internal_handle.strong_count(), 1);
        assert_eq!(internal_handle.weak_count(), 0);
    }

    #[test]
    fn test_upgrade_from_internal_to_strong() {
        let (sender, _receiver) = unbounded();
        let handle = RawHandle::strong(1, sender);
        let internal_handle = handle.downgrade_internal();
        let upgraded_handle = internal_handle.upgrade_strong().unwrap();
        assert_eq!(upgraded_handle.strong_count(), 2);
    }

    #[test]
    fn test_upgrade_from_internal_to_strong_failure() {
        let (sender, _receiver) = unbounded();
        let handle = RawHandle::strong(1, sender);
        let internal_handle = handle.downgrade_internal();
        drop(handle);
        assert!(internal_handle.upgrade_strong().is_none());
    }

    #[test]
    fn test_clone_internal_handle() {
        let (sender, _receiver) = unbounded();
        let handle = RawHandle::strong(1, sender);
        let internal_handle = handle.downgrade_internal();
        let cloned_internal_handle = internal_handle.clone();
        assert_eq!(cloned_internal_handle.strong_count(), 1);
        assert_eq!(cloned_internal_handle.weak_count(), 0);
    }

    #[test]
    #[should_panic(expected = "Internal is weaker than weak, can't downgrade")]
    fn test_downgrade_internal_to_weak_panic() {
        let (sender, _receiver) = unbounded();
        let handle = RawHandle::strong(1, sender);
        let internal_handle = handle.downgrade_internal();
        internal_handle.downgrade_weak();
    }

    #[test]
    fn test_drop_internal_handle() {
        let (sender, receiver) = unbounded();
        let handle = RawHandle::strong(1, sender.clone()); 
        {
            let _internal_handle = handle.downgrade_internal();
        }
        assert!(receiver.try_recv().is_err());
        assert!(handle.strong_count() == 1);
        assert!(handle.weak_count() == 0);
    }

    #[test]
    fn test_drop_last_internal_handle() {
        let (sender, receiver) = unbounded();
        {
            let handle = RawHandle::strong(1, sender.clone());
            let _internal_handle = handle.downgrade_internal();
        }
        assert_eq!(receiver.try_recv().unwrap(), RefMessage::Unload(1));
    }
    #[test]
    fn test_multiple_internal_drop() {
        let (sender, receiver) = unbounded();
        {
            let handle = RawHandle::strong(1, sender.clone());
            let internal_handle1 = handle.downgrade_internal();
            let _internal_handle2 = internal_handle1.clone();
        }
        assert_eq!(receiver.try_recv().unwrap(), RefMessage::Unload(1));
    }
}
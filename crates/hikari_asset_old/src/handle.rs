use std::marker::PhantomData;

pub type HandleIndex = usize;
#[derive(Clone, Copy, Debug)]
pub(crate) enum RefOp {
    Increment(HandleIndex),
    Decrement(HandleIndex),
}
#[derive(Debug, Clone)]
pub(crate) enum RefType {
    Strong(flume::Sender<RefOp>),
    Weak(flume::Sender<RefOp>),
    Internal,
}
#[derive(Default)]
pub(crate) struct RefCounter {
    state: Vec<Option<usize>>,
}

impl RefCounter {
    fn ensure_length(&mut self, ix: usize) {
        if ix >= self.state.len() {
            self.state.resize_with(ix + 1, || None);
        }
    }
    pub fn process_op(&mut self, op: RefOp) {
        match op {
            RefOp::Increment(index) => {
                self.ensure_length(index);
                if let Some(count) = &mut self.state[index] {
                    *count += 1;
                } else {
                    self.state[index] = Some(1);
                }
            }
            RefOp::Decrement(index) => {
                self.ensure_length(index);

                if let Some(count) = &mut self.state[index] {
                    assert!(*count > 0);
                    *count -= 1;
                } else {
                    panic!("Cannot dec_ref on handle index which doesn't exist");
                }
            }
        }
    }
    /// Removes all handles with refcount equal to 0 with the provided closure, passing the handle index(which will be removed) as an argument
    pub fn remove_with(&mut self, mut fun: impl FnMut(HandleIndex)) {
        self.state
            .iter_mut()
            .enumerate()
            .filter(|(_, ref_count)| ref_count.is_some())
            .for_each(|(index, maybe_ref_count)| {
                let mut remove = false;
                if let Some(ref_count) = maybe_ref_count {
                    if *ref_count == 0 {
                        remove = true;
                        (fun)(index);
                    }
                }

                if remove {
                    *maybe_ref_count = None;
                }
            });
    }
}

#[derive(Debug)]
pub struct HandleInner {
    index: HandleIndex,
    ref_type: RefType,
}
impl HandleInner {
    pub(crate) fn new(index: HandleIndex, ref_type: RefType) -> Self {
        match &ref_type {
            RefType::Strong(sender) => sender
                .send(RefOp::Increment(index))
                .expect("Failed to increment refcount"),
            _ => {}
        }
        Self { index, ref_type }
    }
    pub fn is_weak(&self) -> bool {
        matches!(self.ref_type, RefType::Weak(_))
    }
    pub fn clone_weak(&self) -> HandleInner {
        let ref_type = match &self.ref_type {
            RefType::Strong(channel) => RefType::Weak(channel.clone()),
            RefType::Weak(_) => panic!("Handle is already weak"),
            _ => panic!(),
        };

        HandleInner::new(self.index, ref_type)
    }
    pub fn clone_strong(&self) -> HandleInner {
        let ref_type = match &self.ref_type {
            RefType::Strong(_) => panic!("Handle is already strong"),
            RefType::Weak(channel) => RefType::Strong(channel.clone()),
            RefType::Internal => panic!(),
        };

        HandleInner::new(self.index, ref_type)
    }
}

impl Clone for HandleInner {
    fn clone(&self) -> Self {
        match &self.ref_type {
            RefType::Strong(sender) => {
                sender
                    .send(RefOp::Increment(self.index))
                    .expect("Failed to increment refcount");
                println!("Incref {}", self.index);
            }
            _ => {}
        }
        Self {
            index: self.index,
            ref_type: self.ref_type.clone(),
        }
    }
}
impl PartialEq for HandleInner {
    fn eq(&self, other: &Self) -> bool {
        self.index == other.index
    }
}
impl Eq for HandleInner {}

impl std::hash::Hash for HandleInner {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.index.hash(state);
    }
}

impl Drop for HandleInner {
    fn drop(&mut self) {
        println!("Decref {}", self.index);
        match &self.ref_type {
            RefType::Strong(sender) => sender
                .send(RefOp::Decrement(self.index))
                .expect("Failed to decrement refcount"),
            _ => {}
        }
    }
}

pub struct Handle<T> {
    inner: HandleInner,
    _phantom: PhantomData<T>,
}

impl<T> Handle<T> {
    #[inline]
    pub(crate) fn new(index: HandleIndex, channel: flume::Sender<RefOp>) -> Self {
        let inner = HandleInner::new(index, RefType::Strong(channel));
        Self::from_inner(inner)
    }
    fn from_inner(inner: HandleInner) -> Handle<T> {
        Self {
            inner: inner,
            _phantom: Default::default(),
        }
    }
    #[inline(always)]
    pub fn index(&self) -> HandleIndex {
        self.inner.index
    }
    pub fn is_weak(&self) -> bool {
        self.inner.is_weak()
    }
    pub fn clone_weak(&self) -> Handle<T> {
        Self {
            inner: self.inner.clone_weak(),
            _phantom: Default::default(),
        }
    }
    pub fn clone_strong(&self) -> Handle<T> {
        Self {
            inner: self.inner.clone_strong(),
            _phantom: Default::default(),
        }
    }
}

impl<T> Clone for Handle<T> {
    #[inline]
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            _phantom: self._phantom,
        }
    }
}
impl<T> PartialEq for Handle<T> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.inner == other.inner
    }
}
impl<T> Eq for Handle<T> {}

impl<T> std::hash::Hash for Handle<T> {
    #[inline]
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.inner.hash(state);
    }
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct ErasedHandle {
    inner: HandleInner,
    ty: std::any::TypeId,
}

impl ErasedHandle {
    #[inline(always)]
    pub fn index(&self) -> HandleIndex {
        self.inner.index
    }
    pub fn into_typed<T: 'static>(self) -> Option<Handle<T>> {
        if self.ty == std::any::TypeId::of::<T>() {
            Some(Handle {
                inner: self.inner,
                _phantom: Default::default(),
            })
        } else {
            None
        }
    }
    pub fn is_weak(&self) -> bool {
        self.inner.is_weak()
    }
    pub fn clone_weak(&self) -> ErasedHandle {
        Self {
            inner: self.inner.clone_weak(),
            ty: self.ty,
        }
    }

    pub(crate) fn use_for_hashing<T: 'static>(index: HandleIndex) -> ErasedHandle {
        Self {
            inner: HandleInner {
                index,
                ref_type: RefType::Internal,
            },
            ty: std::any::TypeId::of::<T>(),
        }
    }
}

impl<T: 'static> From<Handle<T>> for ErasedHandle {
    fn from(handle: Handle<T>) -> Self {
        ErasedHandle {
            inner: handle.inner,
            ty: std::any::TypeId::of::<T>(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::*;
    fn collect_garbage(
        assets: &mut Assets<u32>,
        recv: &flume::Receiver<RefOp>,
        refcounter: &mut RefCounter,
    ) {
        for op in recv.try_iter() {
            refcounter.process_op(op);
        }
        refcounter.remove_with(|index| {
            println!("Removing  {index}");
            assets.remove(index);
        });
    }
    #[test]
    fn handle_lifetimes() {
        let mut assets = Assets::new();
        let mut refcounter = RefCounter::default();
        let handle_allocator = assets.handle_allocator().clone();
        let recv = handle_allocator.ref_op_channel();

        {
            // let handle = handle_allocator.allocate::<u32>();
            // let other_handle = handle.clone();
            // let other_handle = handle.make_weak();
            // assets.insert(handle.index(), 0);

            let handle = handle_allocator.allocate::<u32>();
            let other_handle = handle.clone();
            let other_handle = handle.clone_weak();
            let _ = other_handle.clone_strong();

            assets.insert(handle.index(), 1);
        }

        for _ in 0..100 {
            collect_garbage(&mut assets, recv, &mut refcounter);
        }
    }
}

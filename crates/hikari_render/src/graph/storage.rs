use std::any::TypeId;
use std::marker::PhantomData;

/// An opaque handle to resources used by the Graph
#[derive(Copy)]
pub struct Handle<T> {
    pub(crate) id: usize,
    _phantom: PhantomData<T>,
}
impl<T> Clone for Handle<T> {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            _phantom: self._phantom,
        }
    }
}
impl<T> std::hash::Hash for Handle<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl<T> PartialEq for Handle<T> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}
impl<T> Eq for Handle<T> {}

#[allow(clippy::from_over_into)]
impl<T: 'static> Into<ErasedHandle> for Handle<T> {
    fn into(self) -> ErasedHandle {
        ErasedHandle {
            id: self.id,
            type_id: TypeId::of::<T>(),
        }
    }
}

use std::fmt::Debug;

use crate::texture::SampledImage;

use super::ImageSize;

impl<T> Debug for Handle<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Handle").field("id", &self.id).finish()
    }
}

impl<T> Handle<T> {
    pub(crate) fn new(id: usize) -> Self {
        Self {
            id,
            _phantom: PhantomData::default(),
        }
    }
}

#[derive(Copy, Hash, PartialEq, Eq)]
pub struct ErasedHandle {
    pub(crate) id: usize,
    pub(crate) type_id: TypeId,
}

impl ErasedHandle {
    #[inline]
    pub fn is_of_type<T: 'static>(&self) -> bool {
        self.type_id == TypeId::of::<T>()
    }
    pub fn into_typed<T: 'static>(self) -> Option<Handle<T>> {
        if self.is_of_type::<T>() {
            Some(Handle::new(self.id))
        } else {
            None
        }
    }
}

impl Clone for ErasedHandle {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            type_id: self.type_id,
        }
    }
}

pub trait Resource: 'static {
    type Metadata: 'static;
}

impl Resource for SampledImage {
    type Metadata = ImageSize;
}

pub struct ResourceList<T: Resource> {
    inner: Vec<(T, T::Metadata)>,
}
impl<T: Resource> ResourceList<T> {
    pub fn new() -> Self {
        Self { inner: Vec::new() }
    }
    #[inline(always)]
    pub fn get(&self, handle: &Handle<T>) -> Option<&(T, T::Metadata)> {
        self.inner.get(handle.id)
    }
    #[inline(always)]
    pub fn get_mut(&mut self, handle: &Handle<T>) -> Option<&mut (T, T::Metadata)> {
        self.inner.get_mut(handle.id)
    }

    pub fn handle_iter(&self) -> HandleIter<T> {
        HandleIter {
            ix: 0,
            len: self.inner.len(),
            _phantom: PhantomData::default(),
        }
    }
}
pub struct HandleIter<'a, T> {
    ix: usize,
    len: usize,
    _phantom: PhantomData<&'a T>,
}
impl<'a, T> Iterator for HandleIter<'a, T> {
    type Item = Handle<T>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.ix < self.len {
            let handle = Handle::new(self.ix);
            self.ix += 1;
            Some(handle)
        } else {
            None
        }
    }
}

pub struct Storage<T: Resource> {
    inner: Vec<(T, T::Metadata)>,
}

impl<T: Resource> Storage<T> {
    pub fn new() -> Self {
        Self { inner: Vec::new() }
    }
    pub fn add(&mut self, data: T, metadata: T::Metadata) -> Handle<T> {
        let id = self.inner.len();
        self.inner.push((data, metadata));

        Handle::new(id)
    }

    #[inline]
    pub fn get(&self, handle: &Handle<T>) -> Option<&T> {
        self.inner.get(handle.id).map(|(data, _)| data)
    }
    #[inline]
    pub fn get_mut(&mut self, handle: &Handle<T>) -> Option<&mut T> {
        self.inner.get_mut(handle.id).map(|(data, _)| data)
    }
    #[inline]
    pub fn get_with_metadata(&self, handle: &Handle<T>) -> Option<(&T, &T::Metadata)> {
        self.inner
            .get(handle.id)
            .map(|(data, metadata)| (data, metadata))
    }
    #[inline]
    pub fn get_with_metadata_mut(
        &mut self,
        handle: &Handle<T>,
    ) -> Option<(&mut T, &mut T::Metadata)> {
        self.inner
            .get_mut(handle.id)
            .map(|(data, metadata)| (data, metadata))
    }

    pub fn replace(
        &mut self,
        handle: &Handle<T>,
        new_data: T,
        new_metadata: T::Metadata,
    ) -> Option<(T, T::Metadata)> {
        if let Some((existing_data, existing_metadata)) = self.get_with_metadata_mut(handle) {
            let old_data = std::mem::replace(existing_data, new_data);
            let old_metadata = std::mem::replace(existing_metadata, new_metadata);

            Some((old_data, old_metadata))
        } else {
            None
        }
    }
}

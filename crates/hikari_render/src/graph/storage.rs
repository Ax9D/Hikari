use std::any::{TypeId, Any};
use std::collections::HashMap;
use std::marker::PhantomData;

/// An opaque handle to resources used by the Graph
pub struct GpuHandle<T> {
    pub(crate) id: usize,
    _phantom: PhantomData<T>,
}
impl<T> Clone for GpuHandle<T> {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            _phantom: self._phantom,
        }
    }
}
impl<T> Copy for GpuHandle<T> {

}
impl<T> std::hash::Hash for GpuHandle<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl<T> PartialEq for GpuHandle<T> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}
impl<T> Eq for GpuHandle<T> {}

#[allow(clippy::from_over_into)]
impl<T: 'static> Into<ErasedHandle> for GpuHandle<T> {
    fn into(self) -> ErasedHandle {
        ErasedHandle {
            id: self.id,
            type_id: TypeId::of::<T>(),
        }
    }
}

use std::fmt::Debug;

use crate::Buffer;
use crate::texture::SampledImage;

use super::ImageSize;

impl<T> Debug for GpuHandle<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Handle").field("id", &self.id).finish()
    }
}

impl<T> GpuHandle<T> {
    pub(crate) fn new(id: usize) -> Self {
        Self {
            id,
            _phantom: PhantomData::default(),
        }
    }
}

#[derive(Debug, Copy, Hash, PartialEq, Eq)]
pub struct ErasedHandle {
    pub(crate) id: usize,
    pub(crate) type_id: TypeId,
}

impl ErasedHandle {
    #[inline]
    pub fn is_of_type<T: 'static>(&self) -> bool {
        self.type_id == TypeId::of::<T>()
    }
    pub fn into_typed<T: 'static>(self) -> Option<GpuHandle<T>> {
        if self.is_of_type::<T>() {
            Some(GpuHandle::new(self.id))
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
impl<B: Buffer + 'static> Resource for B {
    type Metadata = ();
}

pub struct ResourceList<T: Resource> {
    inner: Vec<(T, T::Metadata)>,
}
impl<T: Resource> ResourceList<T> {
    pub fn new() -> Self {
        Self { inner: Vec::new() }
    }
    #[inline(always)]
    pub fn get(&self, handle: &GpuHandle<T>) -> Option<&(T, T::Metadata)> {
        self.inner.get(handle.id)
    }
    #[inline(always)]
    pub fn get_mut(&mut self, handle: &GpuHandle<T>) -> Option<&mut (T, T::Metadata)> {
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
    type Item = GpuHandle<T>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.ix < self.len {
            let handle = GpuHandle::new(self.ix);
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
    pub fn add(&mut self, data: T, metadata: T::Metadata) -> GpuHandle<T> {
        let id = self.inner.len();
        self.inner.push((data, metadata));

        GpuHandle::new(id)
    }

    #[inline]
    pub fn get(&self, handle: &GpuHandle<T>) -> Option<&T> {
        self.inner.get(handle.id).map(|(data, _)| data)
    }
    #[inline]
    pub fn get_from_erased(&self, handle: &ErasedHandle) -> Option<&T> {
        self.inner.get(handle.id).map(|(data, _)| data)
    }
    #[inline]
    pub fn get_mut(&mut self, handle: &GpuHandle<T>) -> Option<&mut T> {
        self.inner.get_mut(handle.id).map(|(data, _)| data)
    }
    #[inline]
    pub fn get_from_erased_mut(&mut self, handle: &ErasedHandle) -> Option<&mut T> {
        self.inner.get_mut(handle.id).map(|(data, _)| data)
    }
    #[inline]
    pub fn get_with_metadata(&self, handle: &GpuHandle<T>) -> Option<(&T, &T::Metadata)> {
        self.inner
            .get(handle.id)
            .map(|(data, metadata)| (data, metadata))
    }
    #[inline]
    pub fn get_with_metadata_mut(
        &mut self,
        handle: &GpuHandle<T>,
    ) -> Option<(&mut T, &mut T::Metadata)> {
        self.inner
            .get_mut(handle.id)
            .map(|(data, metadata)| (data, metadata))
    }

    pub fn replace(
        &mut self,
        handle: &GpuHandle<T>,
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

type GenericStorage = HashMap<TypeId, Box<dyn Any + Send + Sync + 'static>>;
pub struct GenericBufferStorage {
    storages: GenericStorage,
    untyped_fetchers: HashMap<TypeId, Box<dyn for<'a> Fn(&'a GenericStorage, &ErasedHandle) -> Option<&'a dyn Buffer> + Send + Sync>>
}

impl GenericBufferStorage {
    pub fn new() -> Self {
        Self {
            storages: Default::default(),
            untyped_fetchers: Default::default()
        }
    }
    pub fn add<B: Buffer + Send + Sync + 'static>(&mut self, data: B) -> GpuHandle<B> {
        // let storage = self.storages.entry(TypeId::of::<B>()).or_insert_with(|| Box::new(Storage::<B>::new()));
        
        if let Some(storage) = self.storages.get_mut(&TypeId::of::<B>()) {
            let storage = storage.downcast_mut::<Storage<B>>().unwrap();
            return storage.add(data, ());
        }

        let mut storage = Storage::<B>::new();
        let handle = storage.add(data, ());
        self.storages.insert(TypeId::of::<B>(), Box::new(storage));  

        self.untyped_fetchers.insert(TypeId::of::<B>(), Box::new(|storage, handle| -> Option<&dyn Buffer> {
            let storage = storage.get(&handle.type_id).unwrap();
            let storage = storage.downcast_ref::<Storage<B>>().unwrap();

            storage.get_from_erased(handle).map(|buffer| {
                let downcasted: &dyn Buffer = buffer;

                downcasted
            })
        }));
        
        handle
    }
    pub fn get<B: Buffer + Send + Sync + 'static>(&self, handle: &GpuHandle<B>) -> Option<&B> {
        let storage = self.storages.get(&TypeId::of::<B>())?;
        let storage = storage.downcast_ref::<Storage<B>>().unwrap();
        storage.get(handle)
    }
    pub fn get_mut<B: Buffer + Send + Sync + 'static>(&mut self, handle: &GpuHandle<B>) -> Option<&mut B> {
        let storage = self.storages.get_mut(&TypeId::of::<B>())?;
        let storage = storage.downcast_mut::<Storage<B>>().unwrap();
        storage.get_mut(handle)
    }
    pub fn get_dyn_buffer(&self, handle: &ErasedHandle) -> Option<&dyn Buffer> {
        let fetcher = self.untyped_fetchers.get(&handle.type_id)?;

        (fetcher)(&self.storages, handle)
    }
}
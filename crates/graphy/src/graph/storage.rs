use std::marker::PhantomData;

#[derive(Clone, Hash, PartialEq, Eq)]
pub struct GpuHandle<T> {
    id: usize,
    _phantom: PhantomData<T>
}

impl<T> GpuHandle<T> {
    pub(crate) fn new(id: usize) -> Self {
        Self {
            id,
            _phantom: PhantomData::default()
        }
    }
}
pub struct Storage<T> {
    inner: Vec<T>
}

impl<T> Storage<T> {
    pub fn add(&mut self, data: T) -> GpuHandle<T> {
        let id = self.inner.len();
        self.inner.push(data);

        GpuHandle::new(id)
    } 
    #[inline]
    pub fn get(&mut self, handle: GpuHandle<T>) -> Option<&T> {
        self.inner.get(handle.id)
    }
    #[inline]
    pub fn get_mut(&mut self, handle: GpuHandle<T>) -> Option<&mut T> {
        self.inner.get_mut(handle.id)
    }

    pub fn replace(&mut self, handle: GpuHandle<T>, new_data: T) -> Option<T> {
        if let Some(existing_data) = self.get_mut(handle) {
            let old_data = std::mem::replace(existing_data, new_data);

            Some(old_data)
        } else {
            None
        }
    }
}

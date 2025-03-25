use std::any::TypeId;

use crate::{Asset, Handle};

use hikari_handle::RawHandle;

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct ErasedHandle {
    raw: RawHandle,
    type_id: TypeId,
}

impl ErasedHandle {
    pub fn from_raw<T: Asset>(raw: RawHandle) -> Self {
        Self {
            raw,
            type_id: TypeId::of::<T>()
        }
    }
    pub fn type_id_asset(&self) -> TypeId {
        self.type_id
    }
    pub fn index(&self) -> usize {
        self.raw().index()
    }
    pub fn raw(&self) -> &RawHandle {
        &self.raw
    }
    pub fn into_typed<T: Asset>(self) -> Option<Handle<T>> {
        if TypeId::of::<T>() == self.type_id {
            Some(Handle::from_raw(self.raw))
        } else {
            None
        }
    }
    pub fn clone_typed<T: Asset>(&self) -> Option<Handle<T>> {
        self.clone().into_typed::<T>()
    }
    fn clone(&self) -> Self {
        Self {
            raw: self.raw.clone(),
            type_id: self.type_id,
        }
    }
    pub fn upgrade_strong(&self) -> Option<Self> {
        Some(Self {
            raw: self.raw.upgrade_strong()?,
            type_id: self.type_id
        })
    }
    pub(crate) fn upgrade_strong_anyway(&self) -> Self {
        Self {
            raw: self.raw.__upgrade_strong_anyway(),
            type_id: self.type_id
        }
    }
    pub(crate) fn upgrade_weak(&self) -> Self {
        Self {
            raw: self.raw.upgrade_weak(),
            type_id: self.type_id
        }
    }
    pub fn clone_weak(&self) -> Self {
        Self {
            raw: self.raw.downgrade_weak(),
            type_id: self.type_id,
        }
    }
    pub fn to_weak(self) -> Self {
        Self {
            raw: self.raw.downgrade_weak(),
            type_id: self.type_id
        }
    }
    pub fn strong_count(&self) -> usize {
        self.raw.strong_count()
    }
    pub fn weak_count(&self) -> usize {
        self.raw.weak_count()
    }
    pub fn is_weak(&self) -> bool {
        self.raw.is_weak()
    }
    pub fn is_internal(&self) -> bool {
        self.raw.is_internal()
    }
}


// impl Borrow<(TypeId, usize)> for ErasedHandle {
//     fn borrow(&self) -> &(TypeId, usize) {
//         &(self.type_id, self.index())
//     }
// }
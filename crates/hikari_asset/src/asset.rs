use std::{cell::UnsafeCell, path::Path, sync::Arc};

// #[derive(Clone)]
// pub struct AssetInternal<T> {
//     inner: Arc<T>,
//     path: std::path::PathBuf,
// }
// #[derive(Clone)]
// pub struct Asset<T> {
//     inner: AssetInternal<T>,
// }
// impl<T> Asset<T> {
//     pub fn new(asset: T, path: &Path) -> Asset<T> {
//         Asset {
//             inner: AssetInternal {
//                 inner: Arc::new(asset),
//                 path: path.to_owned(),
//             },
//         }
//     }
// }

// impl<T> std::ops::Deref for AssetInternal<T> {
//     type Target = T;

//     fn deref(&self) -> &Self::Target {
//         &self.inner
//     }
// }

// impl<T> std::ops::DerefMut for AssetInternal<T> {
//     fn deref_mut(&mut self) -> &mut Self::Target {
//         unsafe { &mut *(Arc::as_ptr(&self.inner) as *mut T) }
//     }
// }
pub struct AssetData {
    name: String,
    path: std::path::PathBuf,
}
pub struct AssetInternal<T> {
    asset_data: AssetData,
    data: T,
}
unsafe impl<T> Send for Asset<T> {}
unsafe impl<T> Sync for Asset<T> {}

pub struct Asset<T> {
    inner: Arc<UnsafeCell<AssetInternal<T>>>,
}
pub use std::ops::Deref;
pub use std::ops::DerefMut;

impl<T> Deref for Asset<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &(unsafe { &*self.inner.get() }).data
    }
}
impl<T> DerefMut for Asset<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut (unsafe { &mut *self.inner.get() }).data
    }
}
impl<T> Clone for Asset<T> {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
        }
    }
}

#[inline]
fn mut_ptr_to_ref<'a, T>(ptr: *mut T) -> &'a T {
    unsafe { &*ptr }
}

impl<T> Asset<T> {
    pub fn name(asset: &Asset<T>) -> &str {
        &mut_ptr_to_ref(asset.inner.get()).asset_data.name
    }
    pub fn path(asset: &Asset<T>) -> &Path {
        &mut_ptr_to_ref(asset.inner.get()).asset_data.path
    }
    pub fn new<P: AsRef<Path>>(name: &str, path: P, data: T) -> Self {
        Self {
            inner: Arc::new(UnsafeCell::new(AssetInternal {
                data,
                asset_data: AssetData {
                    name: name.to_owned(),
                    path: path.as_ref().to_owned(),
                },
            })),
        }
    }
}

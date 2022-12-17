use std::{
    any::{Any, TypeId},
    hash::Hasher,
};

use fxhash::FxHasher;
use imgui::Ui;
use nohash_hasher::IntMap;
use once_cell::sync::OnceCell;
use parking_lot::{Mutex, MutexGuard};

static STORAGE: OnceCell<Mutex<Storage>> = OnceCell::new();

#[derive(Default)]
pub struct Storage {
    id_type_to_data: IntMap<u64, Box<dyn Any + Send + Sync + 'static>>,
}
impl Storage {
    pub fn get_or_insert_with<T: Any + Send + Sync>(
        &mut self,
        id: imgui::Id,
        default: impl FnOnce() -> T,
    ) -> &mut T {
        let hash = hash(id, TypeId::of::<T>());

        let data = self.id_type_to_data.entry(hash).or_insert_with(|| {
            let data = (default)();
            Box::new(data)
        });

        data.downcast_mut::<T>().unwrap()
    }
    pub fn insert<T: Any + Send + Sync>(&mut self, id: imgui::Id, data: T) -> Option<T> {
        let hash = hash(id, TypeId::of::<T>());

        self.id_type_to_data.insert(hash, Box::new(data)).map(|any| *any.downcast::<T>().unwrap())
    }
    pub fn get<T: Any + Send + Sync>(&mut self,
    id: imgui::Id,
    ) -> Option<&T> {
        let hash = hash(id, TypeId::of::<T>());

        let data = self.id_type_to_data.get(&hash);

        data.map(|any| any.downcast_ref::<T>().unwrap())
    }
}
pub trait StorageExt {
    fn storage(&self) -> MutexGuard<'_, Storage>;
}

fn get_storage<'a>() -> MutexGuard<'a, Storage> {
    STORAGE
        .get_or_init(|| Mutex::new(Storage::default()))
        .lock()
}
fn hash(id: imgui::Id, type_id: TypeId) -> u64 {
    use std::hash::Hash;
    let mut state = FxHasher::default();
    id.type_id().hash(&mut state);
    type_id.hash(&mut state);

    state.finish()
}
impl StorageExt for Ui {
    fn storage(&self) -> MutexGuard<'_, Storage> {
        get_storage()
    }
}

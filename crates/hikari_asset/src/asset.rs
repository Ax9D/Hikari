use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};
use type_uuid::TypeUuid;
use uuid::Uuid;

use crate::{ErasedHandle, Record};

pub trait Asset: Send + Sync + 'static {
    type Settings: Send
        + Sync
        + Default
        + Clone
        + TypeUuid
        + Serialize
        + for<'a> Deserialize<'a>
        + 'static;
}

#[derive(Default)]
pub struct AssetDB {
    pub(crate) records: Vec<Record>,
    uuid_to_record: HashMap<Uuid, usize, fxhash::FxBuildHasher>,
    path_to_record: HashMap<PathBuf, usize, fxhash::FxBuildHasher>,

    uuid_to_handle: HashMap<Uuid, ErasedHandle, fxhash::FxBuildHasher>,
    handle_to_record: HashMap<ErasedHandle, usize, fxhash::FxBuildHasher>,
}

impl AssetDB {
    pub fn new() -> Self {
        Self::default()
    }
    pub(crate) fn build_query_accelerators(&mut self) {
        for (ix, record) in self.records.iter().enumerate() {
            self.uuid_to_record.insert(record.uuid.clone(), ix);
            self.path_to_record.insert(record.path.clone(), ix);
        }
    }
    fn create_links(&mut self, handle: ErasedHandle, record_index: usize) {
        let record = &self.records[record_index];

        self.handle_to_record.insert(handle.clone(), record_index);
        self.uuid_to_record.insert(record.uuid, record_index);
        self.path_to_record
            .insert(record.path.clone(), record_index);
        self.uuid_to_handle.insert(record.uuid, handle);
    }
    fn register_new_handle<T: Asset>(
        &mut self,
        handle: ErasedHandle,
        path: PathBuf,
        settings: T::Settings,
    ) {
        assert!(handle.is_weak());

        let uuid = Uuid::new_v4();
        let record = Record::new::<T>(uuid, path.clone(), settings);

        let record_index = self.records.len();
        self.records.push(record);

        self.create_links(handle, record_index);
    }
    pub(crate) fn assign_handle<T: Asset>(
        &mut self,
        handle: &ErasedHandle,
        path: PathBuf,
        settings: T::Settings,
    ) {
        match self.path_to_record.get(&path) {
            Some(&index) => {
                self.create_links(handle.clone(), index);
            }
            None => self.register_new_handle::<T>(handle.clone(), path, settings),
        }
    }
    pub(crate) fn fix_uuid(&mut self, current: &Uuid, fixed: Uuid) -> Option<()> {
        let record_ix = self.uuid_to_record.get(current)?;
        self.records[*record_ix].uuid = fixed;

        let removed = self.uuid_to_handle.remove(current);
        if let Some(removed) = removed {
            self.uuid_to_handle.insert(fixed, removed);
        }

        Some(())
    }
    #[allow(unused)]
    pub(crate) fn remove_handle(&mut self, handle: ErasedHandle) -> anyhow::Result<()> {
        assert!(handle.is_weak());

        let Some(&record_index) = self.handle_to_record.get(&handle) else {
            return Err(anyhow::anyhow!("Handle was not registered"));
        };

        let removed_record = self.records.swap_remove(record_index);

        //Remove references to old record
        self.handle_to_record.remove(&handle);
        self.uuid_to_record.remove(&removed_record.uuid);
        self.path_to_record.remove(&removed_record.path);
        self.uuid_to_handle.remove(&removed_record.uuid);

        if self.records.len() > 0 {
            //Update references to the moved record
            let moved_record = &self.records[record_index];
            let moved_record_handle = &self.uuid_to_handle[&moved_record.uuid];

            *self.handle_to_record.get_mut(&moved_record_handle).unwrap() = record_index;
            *self.uuid_to_record.get_mut(&moved_record.uuid).unwrap() = record_index;
            *self.path_to_record.get_mut(&moved_record.path).unwrap() = record_index;
        }

        Ok(())
    }
    pub(crate) fn rename_record(&mut self, path: &Path, new_path: &Path) -> anyhow::Result<()> {
        let record_ix = self
            .path_to_record
            .remove(path)
            .ok_or(anyhow::anyhow!("Cannot rename record: Path doesn't exist!"))?;
        self.path_to_record.insert(new_path.to_owned(), record_ix);
        Ok(())
    }
    pub fn uuid_to_record(&self, uuid: &Uuid) -> Option<&Record> {
        Some(&self.records[*self.uuid_to_record.get(uuid)?])
    }
    pub fn handle_to_record(&self, handle: &ErasedHandle) -> Option<&Record> {
        self.handle_to_record
            .get(handle)
            .map(|&index| &self.records[index])
    }
    pub fn handle_to_record_mut(&mut self, handle: &ErasedHandle) -> Option<&mut Record> {
        self.handle_to_record
            .get(handle)
            .map(|&index| &mut self.records[index])
    }
    pub fn handle_to_uuid(&self, handle: &ErasedHandle) -> Option<&Uuid> {
        self.handle_to_record(handle).map(|record| &record.uuid)
    }
    pub fn handle_to_path(&self, handle: &ErasedHandle) -> Option<PathBuf> {
        self.handle_to_record(handle)
            .map(|record| record.path.clone())
    }
    pub fn uuid_to_handle(&self, uuid: &Uuid) -> Option<&ErasedHandle> {
        self.uuid_to_handle.get(uuid)
    }
    pub fn uuid_to_path(&self, uuid: &Uuid) -> Option<&Path> {
        self.uuid_to_record(uuid)
            .map(|record| record.path.as_path())
    }
    pub fn path_to_handle(&self, path: &Path) -> Option<&ErasedHandle> {
        let record = &self.records[*self.path_to_record.get(path)?];
        self.uuid_to_handle(&record.uuid)
    }
    pub fn path_to_handle_and_record(
        &mut self,
        path: &Path,
    ) -> Option<(&ErasedHandle, &mut Record)> {
        let record = &mut self.records[*self.path_to_record.get(path)?];
        let handle = self.uuid_to_handle.get(&record.uuid)?;

        Some((handle, record))
    }
    /// Removes all assets which do not have an handle associated with them
    pub fn clean_unref(&mut self) {
        let uuid_to_handle = &mut self.uuid_to_handle;
        let uuid_to_record = &mut self.uuid_to_record;
        let path_to_record = &mut self.path_to_record;
        //let handle_to_record = &self.handle_to_record;

        self.records.retain(|record| {
            let will_retain = uuid_to_handle.contains_key(&record.uuid);

            if !will_retain {
                uuid_to_handle.remove(&record.uuid);
                uuid_to_record.remove(&record.uuid);
                path_to_record.remove(&record.path);
            }

            will_retain
        });
    }
    /// Returns a list of records in no particular order
    pub fn records(&self) -> &[Record] {
        &self.records
    }
}

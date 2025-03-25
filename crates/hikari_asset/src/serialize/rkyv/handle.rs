use std::path::PathBuf;

use rkyv::{Serialize, ser::Serializer, Archive};
use uuid::Uuid;

use crate::{Asset, Handle};

impl<T: Asset> Archive for Handle<T> {
    type Archived = ArchivedHandle;

    type Resolver = HandleResolver;

    unsafe fn resolve(&self, pos: usize, resolver: Self::Resolver, out: *mut Self::Archived) {
        // let (uuid_p, uuid_o) = rkyv::out_field!(out.uuid);
        // *uuid_o = uuid;
        // let (path_p, path_o) = rkyv::out_field!(out.path);
        // *path_o = path;
        todo!()
    }
}
impl<T: Asset, S: Serializer + ?Sized> Serialize<S> for Handle<T> {
    fn serialize(&self, serializer: &mut S) -> Result<Self::Resolver, S::Error>
    where
        S: Serializer,
    {
        let asset_manager = crate::manager::get_asset_manager();
        let asset_db = asset_manager.asset_db().read();
        let erased = self.clone_erased_as_weak();

        let uuid = *asset_db
        .handle_to_uuid(&erased)
        .expect("Unregistered Handle");
        let path = asset_db
        .handle_to_path(&erased)
        .expect("Unregistered Handle");

        //There's Probably a better way, but this works
        let path_str = path.to_str().unwrap();
        
        todo!()
    }
}

pub struct ArchivedHandle {
    uuid: Uuid,
    path: PathBuf
}
pub struct HandleResolver;
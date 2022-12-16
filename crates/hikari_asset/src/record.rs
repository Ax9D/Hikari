use std::{
    any::{Any},
    path::PathBuf,
};
use type_uuid::TypeUuid;
use uuid::Uuid;

use crate::Asset;

pub struct Record {
    pub uuid: Uuid,
    pub path: PathBuf,
    pub settings_typeid: type_uuid::Bytes,
    pub settings: Box<dyn Any + Send + Sync>,
}

impl Record {
    pub fn new<T: Asset>(uuid: Uuid, path: PathBuf, settings: T::Settings) -> Self {
        let settings = Box::new(settings);
        let settings: Box<dyn Any + Send + Sync> = settings;

        Self {
            uuid,
            path,
            settings_typeid: T::Settings::UUID,
            settings,
        }
    }
    pub fn settings<T: Asset>(&self) -> &T::Settings {
        self.settings.downcast_ref::<T::Settings>().unwrap()
    }
    pub fn settings_mut<T: Asset>(&mut self) -> &mut T::Settings {
        self.settings.downcast_mut::<T::Settings>().unwrap()
    }
}

use std::{any::Any, io::Write, collections::HashSet};

use crate::{Asset, ErasedHandle};

pub struct SaveContext<'a> {
    asset: &'a dyn Any,
}
impl<'a> SaveContext<'a> {
    pub(crate) fn new<T: Asset>(asset: &'a T) -> Self {
        Self { asset: asset }
    }
    pub fn get_asset<T: Asset>(&self) -> &T {
        self.asset.downcast_ref().expect("Incorrect asset type")
    }
}
// pub(crate) struct SaveState<T: Asset> {
//     pub save_queue: Mutex<Vec<SaveTask<T>>>
// }

// impl<T: Asset> SaveState<T> {
//     pub fn new() -> Self {
//         Self { save_queue: Mutex::new(Vec::new()) }
//     }
// }

// impl<T: Asset> SaveState<T> {
//     pub fn enqueue(&self, handle: &Handle<T>) {
//         self.save_queue.lock().push(SaveTask {
//             handle: handle.clone()
//         });
//     }
// }

// pub(crate) struct SaveTask<T> {
//     pub handle: Handle<T>,
// }

pub trait Saver: Send + Sync + 'static {
    fn extensions(&self) -> &[&str];
    fn save(&self, context: &mut SaveContext, writer: &mut dyn Write) -> anyhow::Result<()>;
}

#[derive(Default)]
pub struct Unsaved {
    handles: HashSet<ErasedHandle>
}

impl Unsaved {
    pub fn add_unsaved(&mut self, handle: ErasedHandle) {
        assert!(handle.is_internal());
        self.handles.insert(handle);
    }
    pub fn contains(&self, handle: &ErasedHandle) -> bool {
        self.handles.contains(handle)
    }
    pub fn save(&mut self, handle: &ErasedHandle) -> bool {
        assert!(handle.is_internal());
        self.handles.remove(handle)
    } 
}
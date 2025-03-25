use __core::marker::PhantomData;
use imgui::sys::ImGuiPayload;
use imgui::*;
use std::{any, any::Any, collections::HashMap};

use crate::ImguiInternalExt;
use crate::StorageExt;

pub struct DragDropHandle;

#[derive(Default)]
struct DragDropStorage {
    map: HashMap<String, Box<dyn Any + Send + Sync>>, // HashMap to support multidrag of different payload data types
}

impl DragDropStorage {
    pub fn set_drag_data<T: Any + Send + Sync>(&mut self, payload_name: impl AsRef<str>, data: T) {
        self.map
            .insert(payload_name.as_ref().to_string(), Box::new(data));
    }
    pub fn take_drag_data<T: Any + Send + Sync>(
        &mut self,
        payload_name: impl AsRef<str>,
    ) -> Option<T> {
        self.map.remove(payload_name.as_ref()).map(|any| {
            *any.downcast()
                .expect("Incorrect drag and drop payload type")
        })
    }
}

pub const GLOBAL_DRAG_DROP_STORAGE_ID: &str = "GLOBAL_DRAG_DROP_STORAGE";
pub struct DragDropHelperSource<'ui, L> {
    payload_name: L,
    flags: DragDropFlags,
    cond: Condition,
    ui: &'ui Ui,
}

impl<'ui, L: AsRef<str>> DragDropHelperSource<'ui, L> {
    fn new(ui: &'ui Ui, payload_name: L) -> Self {
        Self {
            payload_name,
            flags: DragDropFlags::empty(),
            cond: Condition::Always,
            ui,
        }
    }
    pub fn flags(mut self, flags: DragDropFlags) -> Self {
        self.flags = flags;

        self
    }
    pub fn condition(mut self, cond: Condition) -> Self {
        self.cond = cond;

        self
    }
    pub fn begin<T: Any + Send + Sync>(self, data: T) -> Option<DragDropSourceHelperToolTip<'ui>> {
        let mut storage = self.ui.storage();
        let drag_drop_storage = storage
            .get_or_insert_with(self.ui.new_id_ptr(&GLOBAL_DRAG_DROP_STORAGE_ID), || {
                DragDropStorage::default()
            });

        let tooltip = unsafe { self.begin_payload() };

        drag_drop_storage.set_drag_data(self.payload_name, data);

        tooltip
    }
    unsafe fn begin_payload(&self) -> Option<DragDropSourceHelperToolTip<'ui>> {
        let should_begin = sys::igBeginDragDropSource(self.flags.bits() as i32);

        //Send a typed payload header for compatibility with imgui-rs api
        let payload = TypedPayloadHeader::new::<()>();
        let payload = &payload as *const _ as *const std::ffi::c_void;

        if should_begin {
            sys::igSetDragDropPayload(
                self.ui.scratch_txt(&self.payload_name),
                payload,
                std::mem::size_of::<TypedPayloadHeader>(),
                self.cond as i32,
            );

            Some(DragDropSourceHelperToolTip::push())
        } else {
            None
        }
    }
}

/// A header for a typed payload for compatibility with existing imgui-rs drag drop api
#[derive(Debug, Clone, Copy, PartialEq, Eq, Ord, PartialOrd)]
#[repr(C)]
struct TypedPayloadHeader {
    type_id: any::TypeId,
    #[cfg(debug_assertions)]
    type_name: &'static str,
}

impl TypedPayloadHeader {
    #[cfg(debug_assertions)]
    fn new<T: 'static>() -> Self {
        Self {
            type_id: any::TypeId::of::<T>(),
            type_name: any::type_name::<T>(),
        }
    }

    #[cfg(not(debug_assertions))]
    fn new<T: 'static>() -> Self {
        Self {
            type_id: any::TypeId::of::<T>(),
        }
    }
}

pub struct DragDropSourceHelperToolTip<'ui>(PhantomData<&'ui Ui>);

impl<'ui> DragDropSourceHelperToolTip<'ui> {
    fn push() -> Self {
        Self(PhantomData)
    }
    pub fn end(self) {}
}
impl<'ui> Drop for DragDropSourceHelperToolTip<'ui> {
    fn drop(&mut self) {
        unsafe { sys::igEndDragDropSource() }
    }
}

#[derive(Debug)]
pub struct DragDropHelperTarget<'ui>(&'ui Ui);

impl<'ui> DragDropHelperTarget<'ui> {
    fn new(ui: &'ui Ui) -> Option<Self> {
        let should_begin = unsafe { sys::igBeginDragDropTarget() };
        if should_begin {
            Some(Self(ui))
        } else {
            None
        }
    }
    pub fn accept_payload<T: Any + Send + Sync>(
        &self,
        payload_name: impl AsRef<str>,
        flags: DragDropFlags,
    ) -> Option<DragDropHelperPayload<T>> {
        let mut storage = self.0.storage();

        let drag_drop_storage = storage
            .get_or_insert_with(self.0.new_id_ptr(&GLOBAL_DRAG_DROP_STORAGE_ID), || {
                DragDropStorage::default()
            });

        let data = drag_drop_storage.take_drag_data::<T>(&payload_name)?;
        let raw_payload = unsafe { self.accept_payload_unchecked(payload_name, flags)? };

        // let received =
        // unsafe { (raw_payload.Data as *const TypedPayloadHeader).read_unaligned() };

        // let expected = TypedPayloadHeader::new::<T>();

        // if received != expected {
        //     return Some(Err(PayloadIsWrongType {
        //         expected,
        //         received
        //     }));
        // }

        Some(DragDropHelperPayload {
            data,
            preview: raw_payload.Preview,
            delivery: raw_payload.Delivery,
        })
    }
    unsafe fn accept_payload_unchecked(
        &self,
        name: impl AsRef<str>,
        flags: DragDropFlags,
    ) -> Option<ImGuiPayload> {
        let inner = sys::igAcceptDragDropPayload(self.0.scratch_txt(name), flags.bits() as i32);
        if inner.is_null() {
            None
        } else {
            let inner = *inner;
            Some(inner)
        }
    }
}
// /// Indicates that an incorrect payload type was received. It is opaque,
// /// but you can view useful information with Debug formatting when
// /// `debug_assertions` are enabled.
// #[derive(Debug, Clone, Copy, PartialEq, Eq, Ord, PartialOrd)]
// pub struct PayloadIsWrongType {
//     expected: TypedPayloadHeader,
//     received: TypedPayloadHeader,
// }

// #[cfg(debug_assertions)]
// impl std::fmt::Display for PayloadIsWrongType {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         write!(
//             f,
//             "Payload is {} -- expected {}",
//             self.received.type_name, self.expected.type_name
//         )
//     }
// }

// #[cfg(not(debug_assertions))]
// impl std::fmt::Display for PayloadIsWrongType {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         f.pad("Payload is wrong type")
//     }
// }

pub struct DragDropHelperPayload<T> {
    pub data: T,
    pub preview: bool,
    pub delivery: bool,
}

pub trait DragDropHelperExt {
    fn drag_drop_helper_source<L: AsRef<str>>(&self, payload_name: L) -> DragDropHelperSource<L>;
    fn drag_drop_helper_target(&self) -> Option<DragDropHelperTarget>;
}

impl DragDropHelperExt for Ui {
    fn drag_drop_helper_source<L: AsRef<str>>(&self, payload_name: L) -> DragDropHelperSource<L> {
        DragDropHelperSource::new(self, payload_name)
    }
    fn drag_drop_helper_target(&self) -> Option<DragDropHelperTarget> {
        DragDropHelperTarget::new(self)
    }
}

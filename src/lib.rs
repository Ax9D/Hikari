#![allow(dead_code)]
#![allow(unused_imports)]

extern crate cupid;

#[macro_use(lazy_static)]
extern crate lazy_static;

pub mod core;
pub use crate::core::init;
pub use glam;

pub mod render;
pub use render::model;
pub use render::model::Model;
pub use render::scenerenderer;
pub use render::texture;

pub mod dev;

pub mod ui;

pub mod input;
pub mod script;
pub mod window;

pub use crate::core::Context;
pub use crate::core::Scene;

#[macro_export]
macro_rules! rawToStr {
    ($raw: expr) => {
        unsafe {
            std::ffi::CStr::from_ptr($raw as *const i8)
                .to_str()
                .unwrap()
        }
    };
}
#[macro_export]
macro_rules! strToRaw {
    ($s: expr) => {
        std::ffi::CString::new($s).unwrap().as_ptr()
    };
}

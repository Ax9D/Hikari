use once_cell::sync::OnceCell;
use parking_lot::{Mutex, MutexGuard};
pub use renderdoc::*;

pub struct RenderdocPlugin;

impl hikari_core::Plugin for RenderdocPlugin {
    fn build(self, game: &mut hikari_core::Game) {
        init_renderdoc();
    }
}

static RENDERDOC: OnceCell<Mutex<RenderDoc<V120>>> = OnceCell::new();

fn init_renderdoc() {
    let renderdoc = RenderDoc::<V120>::new().expect("Failed to initialize renderdoc");
    RENDERDOC.get_or_init(|| Mutex::new(renderdoc));
}
fn get_renderdoc() -> MutexGuard<'static, RenderDoc<V120>> {
    RENDERDOC.get().expect("RenderDoc not initialized").lock()
}

fn end_capture() {
    get_renderdoc().end_frame_capture(std::ptr::null(), std::ptr::null())
}

pub struct FrameCapture;

impl FrameCapture {
    #[must_use]
    pub fn new() -> Self {
        let mut renderdoc = get_renderdoc();
        if !renderdoc.is_frame_capturing() {
            renderdoc.start_frame_capture(std::ptr::null(), std::ptr::null())
        }

        Self
    }
}

impl Drop for FrameCapture {
    fn drop(&mut self) {
        let mut renderdoc = get_renderdoc();
        if renderdoc.is_frame_capturing() {
            renderdoc.end_frame_capture(std::ptr::null(), std::ptr::null())
        }
    }
}

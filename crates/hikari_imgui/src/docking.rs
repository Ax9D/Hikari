use imgui::sys;

pub struct DockNode {
    id: u32
}
impl DockNode {
    pub fn new(id: u32) -> Self {
        Self { id }
    }
    pub fn dock_window(&self, ui: &imgui::Ui, window: &str) {
        let window = ui.scratch_txt(window);
        unsafe { sys::igDockBuilderDockWindow(window, self.id) }
    }
}
pub trait ImguiDockingExt {
    /// Internal method to push a single text to our scratch buffer.
    fn scratch_txt(&self, txt: impl AsRef<str>) -> *const sys::cty::c_char;
    fn dockspace<Label: AsRef<str>>(&self, label: Label) -> DockNode;
    fn dockspace_over_viewport(&self);
}

impl ImguiDockingExt for imgui::Ui {
    /// Internal method to push a single text to our scratch buffer.
    fn scratch_txt(&self, txt: impl AsRef<str>) -> *const sys::cty::c_char {
        unsafe {
            let handle = &mut *self.scratch_buffer().get();
            handle.scratch_txt(txt)
        }
    }
    fn dockspace<Label: AsRef<str>>(&self, label: Label) -> DockNode {
        unsafe {
            let id = sys::igGetIDStr(self.scratch_txt(label));
            sys::igDockSpace(
                id,
                [0.0, 0.0].into(),
                0,
                ::std::ptr::null::<sys::ImGuiWindowClass>(),
            );

            DockNode { id }
        }
    }
    fn dockspace_over_viewport(&self) {
        unsafe {
            sys::igDockSpaceOverViewport(
                sys::igGetMainViewport(),
                0,
                ::std::ptr::null::<sys::ImGuiWindowClass>(),
            );
        }
    }
}

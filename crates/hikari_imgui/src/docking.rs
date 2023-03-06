use crate::ImguiInternalExt;
use imgui::sys;
use imgui::sys::ImGuiDockNodeFlags;
use imgui::Direction;

pub struct DockNode {
    id: u32,
}
impl DockNode {
    pub fn new(id: u32) -> Self {
        Self { id }
    }

    pub fn dock_window(&self, ui: &imgui::Ui, window: impl AsRef<str>) {
        let window = ui.scratch_txt(window);
        unsafe { sys::igDockBuilderDockWindow(window, self.id) }
    }

    pub fn size(self, size: [f32; 2]) -> Self {
        unsafe { sys::igDockBuilderSetNodeSize(self.id, sys::ImVec2::from(size)) }

        self
    }

    pub fn position(self, position: [f32; 2]) -> Self {
        unsafe { sys::igDockBuilderSetNodePos(self.id, sys::ImVec2::from(position)) }

        self
    }

    pub fn split<D: FnOnce(DockNode), O: FnOnce(DockNode)>(
        self,
        split_dir: Direction,
        size_ratio: f32,
        dir: D,
        opposite_dir: O,
    ) {
        let mut out_id_at_dir: sys::ImGuiID = 0;
        let mut out_id_at_opposite_dir: sys::ImGuiID = 0;

        unsafe {
            sys::igDockBuilderSplitNode(
                self.id,
                split_dir as i32,
                size_ratio,
                &mut out_id_at_dir,
                &mut out_id_at_opposite_dir,
            );
        }

        dir(DockNode::new(out_id_at_dir));
        opposite_dir(DockNode::new(out_id_at_opposite_dir));
    }
}

pub trait ImguiDockingExt {
    fn dockspace<Label: AsRef<str>>(
        &self,
        label: Label,
        size: [f32; 2],
        flags: ImGuiDockNodeFlags,
    ) -> DockNode;
    fn dockspace_over_viewport(&self);
    fn docknode<Label: AsRef<str>, F: FnOnce(DockNode)>(
        &self,
        label: Label,
        flags: ImGuiDockNodeFlags,
        f: F,
    );
    fn get_node<Label: AsRef<str>>(&self, label: Label) -> Option<*const sys::ImGuiDockNode>;
}

impl ImguiDockingExt for imgui::Ui {
    fn dockspace<Label: AsRef<str>>(
        &self,
        label: Label,
        size: [f32; 2],
        flags: ImGuiDockNodeFlags,
    ) -> DockNode {
        unsafe {
            let id = sys::igGetID_Str(self.scratch_txt(label));
            sys::igDockSpace(
                id,
                size.into(),
                flags,
                ::std::ptr::null::<sys::ImGuiWindowClass>(),
            );

            DockNode { id }
        }
    }
    fn docknode<Label: AsRef<str>, F: FnOnce(DockNode)>(
        &self,
        label: Label,
        flags: ImGuiDockNodeFlags,
        f: F,
    ) {
        unsafe {
            let id = sys::igGetID_Str(self.scratch_txt(label));

            sys::igDockBuilderRemoveNode(id);

            sys::igDockBuilderAddNode(id, flags);

            f(DockNode::new(id));

            sys::igDockBuilderFinish(id)
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

    fn get_node<Label: AsRef<str>>(&self, label: Label) -> Option<*const sys::ImGuiDockNode> {
        unsafe {
            let id = sys::igGetID_Str(self.scratch_txt(label));
            let root_node_set = sys::igDockBuilderGetNode(id);
            if root_node_set.is_null() {
                None
            } else {
                Some(root_node_set)
            }
        }
    }
}

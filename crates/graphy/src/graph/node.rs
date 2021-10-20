use indexmap::IndexMap;

use crate::ImageSize;

pub(super) struct Node<Scene, PerFrame, Resources> {
    pub name: String,
    pub inputs: IndexMap<String, Input>,
    pub outputs: IndexMap<String, Output>,
    pub pass_kind: PassKind,
    pub draw_fn: Box<dyn Fn(&mut super::CommandBuffer, &Scene, &PerFrame, &Resources)>,
}


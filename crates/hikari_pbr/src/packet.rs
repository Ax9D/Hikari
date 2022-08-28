use crate::resources::RenderResources;

pub struct FramePacket<'res> {
    res: &'res mut RenderResources
}

impl<'res> FramePacket<'res> {
    pub fn new() -> Self {
        Self { res: todo!() }
    }
}

impl<'res> Drop for FramePacket<'res> {
    fn drop(&mut self) {
        todo!()
    }
}
use crate::resources::RenderResources;
#[allow(unused)]
pub struct FramePacket<'res> {
    res: &'res mut RenderResources,
}
#[allow(unused)]
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

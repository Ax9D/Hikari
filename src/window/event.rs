//#[derive(Debug, Copy, Clone)]
// pub enum EventType {
//     Key(Key, Action, Modifiers),
//     MouseClick(MouseButton, Action),
//     MouseMove(f32, f32),
//     Resize(u32, u32),
//     FocusLose,
//     FocusGain,
//     Scroll(f32, f32),
//     Char(char),
//     Unknown,
// }
pub type EventType = glfw::WindowEvent;
pub type Modifiers = glfw::Modifiers;
pub struct Event {
    pub consumed: bool,
    pub kind: EventType,
}
impl Event {
    pub fn new(kind: EventType) -> Self {
        Self {
            consumed: false,
            kind,
        }
    }
    pub fn consume(&mut self) {
        self.consumed = true;
    }
}

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum RefMessage {
    Unload(usize),
    Deallocate(usize)
}

#[derive(Clone)]
pub enum RefType {
    Strong,
    Weak,
    Internal
}
impl RefType {
    pub fn is_strong(&self) -> bool {
        matches!(self, RefType::Strong)
    }
    pub fn is_weak(&self) -> bool {
        matches!(self, RefType::Weak)
    }
    pub fn is_internal(&self) -> bool {
         matches!(self, RefType::Internal)
    }
}
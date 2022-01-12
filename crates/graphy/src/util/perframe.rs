use std::ops::{Deref, DerefMut};

pub struct PerFrame<T, const FRAMES: usize> {
    frames: [T; FRAMES],
    cur_frame: usize,
}

impl<T, const FRAMES: usize> PerFrame<T, FRAMES> {
    pub fn new(frames: [T; FRAMES]) -> Self {
        Self {
            frames,
            cur_frame: 0,
        }
    }
    #[inline]
    pub fn get(&self) -> &T {
        &self.frames[self.cur_frame]
    }
    #[inline]
    pub fn get_mut(&mut self) -> &mut T {
        &mut self.frames[self.cur_frame]
    }
    pub fn next_frame(&mut self) {
        self.cur_frame = self.cur_frame.wrapping_add(1) % FRAMES;
    }
}

impl<T, const FRAMES: usize> Deref for PerFrame<T, FRAMES> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.get()
    }
}
impl<T, const FRAMES: usize> DerefMut for PerFrame<T, FRAMES> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.get_mut()
    }
}

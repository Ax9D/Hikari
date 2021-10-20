use std::sync::atomic::AtomicUsize;

lazy_static! {
    static ref GLOBAL_COUNT: AtomicUsize = AtomicUsize::new(0);
}
use std::ffi::CStr;
use std::ffi::CString;
pub struct Name {
    inner: CString,
}
impl Default for Name {
    fn default() -> Self {
        let x = Self::increment_count();
        Self {
            inner: CString::new(format!("untitled_{}", x)).unwrap(),
        }
    }
}
impl Name {
    fn increment_count() -> usize {
        GLOBAL_COUNT.fetch_add(1, std::sync::atomic::Ordering::SeqCst)
    }
    fn decrement_count() -> usize {
        GLOBAL_COUNT.fetch_sub(1, std::sync::atomic::Ordering::SeqCst)
    }
    pub fn new(name: &str) -> Name {
        Self::increment_count();

        Self {
            inner: CString::new(name).unwrap(),
        }
    }
    pub fn as_cstr(&self) -> &CStr {
        &self.inner.as_c_str()
    }
}
impl Drop for Name {
    fn drop(&mut self) {
        Self::decrement_count();
    }
}

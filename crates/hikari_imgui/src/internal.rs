use imgui::sys;

pub trait ImguiInternalExt {
    /// Internal method to push a single text to our scratch buffer.
    fn scratch_txt(&self, txt: impl AsRef<str>) -> *const sys::cty::c_char;
    /// Internal method to push an option text to our scratch buffer.
    fn scratch_txt_opt(&self, txt: Option<impl AsRef<str>>) -> *const sys::cty::c_char;
    fn scratch_txt_two(
        &self,
        txt_0: impl AsRef<str>,
        txt_1: impl AsRef<str>,
    ) -> (*const sys::cty::c_char, *const sys::cty::c_char);

    fn scratch_txt_with_opt(
        &self,
        txt_0: impl AsRef<str>,
        txt_1: Option<impl AsRef<str>>,
    ) -> (*const sys::cty::c_char, *const sys::cty::c_char);
}

impl ImguiInternalExt for imgui::Ui {
    /// Internal method to push a single text to our scratch buffer.
    fn scratch_txt(&self, txt: impl AsRef<str>) -> *const sys::cty::c_char {
        unsafe {
            let handle = &mut *self.scratch_buffer().get();
            handle.scratch_txt(txt)
        }
    }
    /// Internal method to push an option text to our scratch buffer.
    fn scratch_txt_opt(&self, txt: Option<impl AsRef<str>>) -> *const sys::cty::c_char {
        unsafe {
            let handle = &mut *self.scratch_buffer().get();
            handle.scratch_txt_opt(txt)
        }
    }
    fn scratch_txt_two(
        &self,
        txt_0: impl AsRef<str>,
        txt_1: impl AsRef<str>,
    ) -> (*const sys::cty::c_char, *const sys::cty::c_char) {
        unsafe {
            let handle = &mut *self.scratch_buffer().get();
            handle.scratch_txt_two(txt_0, txt_1)
        }
    }

    fn scratch_txt_with_opt(
        &self,
        txt_0: impl AsRef<str>,
        txt_1: Option<impl AsRef<str>>,
    ) -> (*const sys::cty::c_char, *const sys::cty::c_char) {
        unsafe {
            let handle = &mut *self.scratch_buffer().get();
            handle.scratch_txt_with_opt(txt_0, txt_1)
        }
    }
}

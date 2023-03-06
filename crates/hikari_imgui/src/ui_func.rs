pub trait ImguiUiExt {
    /// Returns `true` if the last item is double clicked
    fn is_double_click(&self, button: imgui::MouseButton) -> bool;
    fn horizontal_align<R>(&self, f: impl FnOnce() -> R, alignment: f32, width: f32) -> R;
    fn full_width(&self, f: impl FnOnce());
}

impl ImguiUiExt for imgui::Ui {
    fn is_double_click(&self, button: imgui::MouseButton) -> bool {
        self.is_item_hovered() && self.is_mouse_double_clicked(button)
    }
    fn horizontal_align<R>(&self, f: impl FnOnce() -> R, alignment: f32, width: f32) -> R {
        let avail = self.content_region_avail()[0];
        let offset = (avail - width) * alignment;

        let cur_pos = self.cursor_pos();
        if offset > 0.0 {
            self.set_cursor_pos([cur_pos[0] + offset, cur_pos[1]]);
        }
        (f)()
    }
    fn full_width(&self, f: impl FnOnce()) {
        let width = self.content_region_avail()[0];
        let _token = self.push_item_width(width);
        (f)();
    }
}

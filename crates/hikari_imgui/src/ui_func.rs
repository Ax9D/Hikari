pub trait ImguiUiExt {
    /// Returns `true` if the last item is double clicked
    fn is_double_click(&self, button: imgui::MouseButton) -> bool;
}

impl ImguiUiExt for imgui::Ui {
    fn is_double_click(&self, button: imgui::MouseButton) -> bool {
        self.is_item_hovered() && self.is_mouse_double_clicked(button)
    }
}

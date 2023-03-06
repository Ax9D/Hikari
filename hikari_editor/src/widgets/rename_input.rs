use hikari::imgui::*;
use hikari::input::KeyCode;

use crate::editor::RenameState;

pub struct RenameInput<'b> {
    id: Id,
    buffer: &'b mut String,
}

impl<'b> RenameInput<'b> {
    pub fn new(id: Id, buffer: &'b mut String) -> Self {
        Self { id, buffer }
    }
    /// Returns the old contents of the buffer when a rename occurs, otherwise returns `None`
    pub fn build(
        mut self,
        ui: &Ui,
        rename_state: &mut RenameState,
        draw_fn: impl FnOnce(&str),
    ) -> Option<String> {
        let mut old_name = None;
        match rename_state {
            RenameState::Renaming(id, current_name, starting_frame) if *id == self.id => {
                let _frame = ui.push_style_var(StyleVar::FramePadding([0.0, 0.0]));

                let input_text_ended = ui
                    .input_text("###rename", current_name)
                    .enter_returns_true(true)
                    .build();

                let end_rename = input_text_ended
                    // If lost focus
                    || !ui.is_window_focused()
                    // If the mouse is clicked on anything but the input text
                    || (ui.is_mouse_clicked(MouseButton::Left) && !ui.is_item_clicked());

                //If the rename was started last frame
                if ui.frame_count() == *starting_frame + 1 {
                    ui.set_keyboard_focus_here();
                }

                //If escape was pressed cancel the rename
                let cancel_rename = ui.io().keys_down[KeyCode::Escape as usize];

                if cancel_rename {
                    *rename_state = RenameState::Idle;
                } else if end_rename {
                    old_name = Some(std::mem::replace::<String>(
                        &mut self.buffer,
                        current_name.clone(),
                    ));
                    *rename_state = RenameState::Idle;
                }
            }
            _ => {
                (draw_fn)(&self.buffer);
            }
        };

        if ui.is_item_focused()
            && (ui.io().keys_down[KeyCode::F2 as usize] || ui.is_double_click(MouseButton::Left))
        {
            match rename_state {
                RenameState::Idle => {
                    *rename_state =
                        RenameState::Renaming(self.id, self.buffer.clone(), ui.frame_count());
                }
                _ => {}
            }
        }

        old_name
    }
}

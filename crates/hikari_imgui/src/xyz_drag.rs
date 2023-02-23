use hikari_math::{Vec3, Vec4};
use imgui::{SliderFlags, Ui, sys, TableColumnSetup, TableFlags, StyleVar, StyleColor};

use crate::ImguiInternalExt;

pub struct DragVec3<L, F = &'static str> {
    label: L,
    speed: f32,
    min: f32,
    max: f32,
    reset: f32,
    width: f32,
    proportional: bool,
    display_format: Option<F>,
    flags: SliderFlags,
}

impl<L: AsRef<str>> DragVec3<L> {
    pub fn new(label: L) -> Self {
        Self {
            label,
            speed: 1.0,
            min: f32::MIN,
            max: f32::MAX,
            display_format: None,
            reset: 0.0,
            width: 0.0,
            proportional: false,
            flags: SliderFlags::empty(),
        }
    }
    /// Sets the range (inclusive)
    pub fn range(mut self, min: f32, max: f32) -> Self {
        self.min = min;
        self.max = max;
        self
    }
    /// Sets the reset value
    pub fn reset(mut self, reset: f32) -> Self {
        self.reset = reset;
        self
    }
    /// Sets the reset value
    pub fn width(mut self, width: f32) -> Self {
        self.width = width;
        self
    }
    pub fn proportional(mut self, lock: bool) -> Self {
        self.proportional = lock;
        self
    }
    /// Sets the value increment for a movement of one pixel.
    ///
    /// Example: speed=0.2 means mouse needs to move 5 pixels to increase the slider value by 1
    pub fn speed(mut self, speed: f32) -> Self {
        self.speed = speed;
        self
    }
    /// Sets the display format using *a C-style printf string*
    pub fn display_format<F2: AsRef<str>>(self, display_format: F2) -> DragVec3<L, F2> {
        DragVec3 {
            label: self.label,
            speed: self.speed,
            min: self.min,
            max: self.max,
            reset: self.reset,
            width: self.width,
            display_format: Some(display_format),
            flags: self.flags,
            proportional: self.proportional,
        }
    }
    /// Replaces all current settings with the given flags
    pub fn flags(mut self, flags: SliderFlags) -> Self {
        self.flags = flags;
        self
    }
    fn build_drag(&self, ui: &Ui, label: impl AsRef<str>, value: &mut f32) -> bool {
        unsafe {
            let (one, two) = ui.scratch_txt_with_opt(label, self.display_format);

            sys::igDragFloat(
                one,
                value as *mut f32,
                self.speed,
                self.min,
                self.max,
                two,
                self.flags.bits() as i32,
            )
        }
    }
    /// Builds a drag slider that is bound to the given value.
    ///
    /// Returns true if the slider value was changed.
    pub fn build(self, ui: &Ui, value: &mut Vec3) -> bool {
        const RED: Vec4 = Vec4::new(0.768, 0.125, 0.125, 1.000);
        const RED_HOVERED: Vec4 = Vec4::new(0.825, 0.275, 0.275, 1.000);
        const RED_CLICKED: Vec4 = Vec4::new(0.618, 0.075, 0.075, 1.000);

        const GREEN: Vec4 = Vec4::new(0.285, 0.634, 0.173, 1.000);
        const GREEN_HOVERED: Vec4 = Vec4::new(0.377, 0.707, 0.270, 1.000);
        const GREEN_CLICKED: Vec4 = Vec4::new(0.216, 0.541, 0.110, 1.000);

        const BLUE: Vec4 = Vec4::new(0.099, 0.348, 0.699, 1.000);
        const BLUE_HOVERED: Vec4 = Vec4::new(0.230, 0.440, 0.736, 1.000);
        const BLUE_CLICKED: Vec4 = Vec4::new(0.093, 0.289, 0.569, 1.000);

        const SPACE: f32 = 5.0;

        let mut changed = false;
        let old = *value;

        if let Some(_token) = ui.begin_table_with_sizing(self.label.as_ref(), 2, TableFlags::SIZING_STRETCH_PROP, [self.width, 0.0], 0.0) {
            ui.table_setup_column_with(TableColumnSetup {
                name: "",
                init_width_or_weight: 15.0,
                ..Default::default()
            });

            ui.table_setup_column_with(TableColumnSetup {
                name: "",
                init_width_or_weight: 85.0,
                ..Default::default()
            });

            ui.table_next_row();
            ui.table_next_column();
            ui.text(self.label.as_ref());
            ui.table_next_column();

            unsafe { sys::igPushMultiItemsWidths(3, ui.calc_item_width()); }


            let _style_token = ui.push_style_var(StyleVar::ItemSpacing([0.0, 0.0]));

            //let line_height = ui.text_line_height();
            //let button_size = [line_height, line_height];

            //X
            {
                let _color = ui.push_style_color(StyleColor::Button, RED);
                let _color = ui.push_style_color(StyleColor::ButtonHovered, RED_HOVERED);
                let _color = ui.push_style_color(StyleColor::ButtonActive, RED_CLICKED);
                let _style_token = ui.push_style_var(StyleVar::FrameRounding(0.0));

                if ui.button("x") {
                    value.x = self.reset;
                    changed = true;
                }
                ui.same_line();
            }
            changed |= self.build_drag(ui, "##X", &mut value.x);

            ui.same_line_with_spacing(0.0, SPACE);
            unsafe {sys::igPopItemWidth()};

            //Y
            {
                let _color = ui.push_style_color(StyleColor::Button, GREEN);
                let _color = ui.push_style_color(StyleColor::ButtonHovered, GREEN_HOVERED);
                let _color = ui.push_style_color(StyleColor::ButtonActive, GREEN_CLICKED);
                let _style_token = ui.push_style_var(StyleVar::FrameRounding(0.0));

                if ui.button("y") {
                    value.y = self.reset;
                    changed = true;
                }
                ui.same_line();
            }
            changed |= self.build_drag(ui, "##Y", &mut value.y);

            ui.same_line_with_spacing(0.0, SPACE);
            unsafe {sys::igPopItemWidth()};

            //Z
            {
                let _color = ui.push_style_color(StyleColor::Button, BLUE);
                let _color = ui.push_style_color(StyleColor::ButtonHovered, BLUE_HOVERED);
                let _color = ui.push_style_color(StyleColor::ButtonActive, BLUE_CLICKED);
                let _style_token = ui.push_style_var(StyleVar::FrameRounding(0.0));

                if ui.button("z") {
                    value.z = self.reset;
                    changed = true;
                }
                ui.same_line();
            }
            changed |= self.build_drag(ui, "##Z", &mut value.z);

            ui.same_line_with_spacing(0.0, SPACE);
            unsafe {sys::igPopItemWidth()};
        }

        if changed && self.proportional {
            let diff = *value - old;
            let mask = diff / (diff.x + diff.y + diff.z);
            let mut ratio = *value / old;

            if !ratio.x.is_finite() {
                ratio.x = 0.0;
            }
            if !ratio.y.is_finite() {
                ratio.y = 0.0;
            }
            if !ratio.z.is_finite() {
                ratio.z = 0.0;
            }

            dbg!(old);
            //dbg!(mask);
            let ratio = ratio * mask;
            let ratio = ratio.x + ratio.y + ratio.z;
            let inv_mask = Vec3::new(1.0, 1.0, 1.0) - mask;
            let mul_mask = inv_mask * ratio + mask;
            let new = *value * mul_mask;
            dbg!(new);
            *value = new;
            dbg!(ratio);
        }

        changed
    }
}
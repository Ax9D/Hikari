use std::f32::consts::TAU;

use hikari_math::{Mat4, Vec3};
use imgui::ImColor32;

use crate::gizmo::math::world_to_screen;

use super::Viewport;

pub(crate) struct Painter3D<'ui> {
    mvp: Mat4,
    viewport: Viewport,
    draw_list: imgui::DrawListMut<'ui>,
}

impl<'ui> Painter3D<'ui> {
    pub fn new(ui: &'ui imgui::Ui, mvp: Mat4, viewport: Viewport) -> Self {
        Self {
            mvp,
            viewport,
            draw_list: ui.get_window_draw_list(),
        }
    }
    pub fn line(&self, from: Vec3, to: Vec3, color: ImColor32, thickness: f32) {
        let from = world_to_screen(self.mvp, self.viewport, from);
        let to = world_to_screen(self.mvp, self.viewport, to);
        if let Some((from, to)) = from.zip(to) {
            self.draw_list
                .add_line(from, to, color)
                .thickness(thickness)
                .build();
        }
    }
    pub fn arrowhead(&self, from: Vec3, to: Vec3, color: ImColor32, thickness: f32) {
        let from = world_to_screen(self.mvp, self.viewport, from);
        let to = world_to_screen(self.mvp, self.viewport, to);

        if let Some((from, to)) = from.zip(to) {
            let direction = (to - from).normalize();
            let right_90 = direction.perp() * thickness;
            let left_90 = -direction.perp() * thickness;

            self.draw_list
                .add_triangle(from + left_90, to, from + right_90, color)
                .filled(true)
                .build();
        }
    }
    // pub fn rect(&self, min: Vec3, max: Vec3, color: ImColor32) {
    //     let min = world_to_screen(self.mvp, self.viewport, min);
    //     let max = world_to_screen(self.mvp, self.viewport, max);

    //     if let Some((min, max)) = min.zip(max) {
    //         let true_min = Vec2::min(min, max);
    //         let true_max = Vec2::max(min, max);
    //         self.draw_list
    //         .add_rect(true_min, true_max, color)
    //         .filled(true)
    //         .build();
    //     }
    // }
    pub fn polygon(&self, points: &[Vec3], color: ImColor32) {
        let points: Vec<_> = points
            .iter()
            .filter_map(|pos| world_to_screen(self.mvp, self.viewport, *pos))
            .collect();

        self.draw_list
            .add_polyline(points, color)
            .filled(true)
            .build();
    }
    pub fn arc(
        &self,
        radius: f32,
        start_angle: f32,
        end_angle: f32,
        color: ImColor32,
        thickness: f32,
    ) {
        let angle = end_angle - start_angle;
        let step_count = steps(angle);
        let mut points = Vec::with_capacity(step_count);

        let step_size = angle / (step_count - 1) as f32;

        for step in (0..step_count).map(|i| step_size * i as f32) {
            let x = f32::cos(start_angle + step) * radius;
            let z = f32::sin(start_angle + step) * radius;

            points.push(Vec3::new(x, 0.0, z));
        }

        let points = points
            .into_iter()
            .filter_map(|point| world_to_screen(self.mvp, self.viewport, point))
            .collect::<Vec<_>>();

        self.draw_list
            .add_polyline(points, color)
            .thickness(thickness)
            .build();
    }
    pub fn circle(&self, radius: f32, color: ImColor32, thickness: f32) {
        self.arc(radius, 0.0, TAU, color, thickness)
    }

    pub fn polyline(&self, points: &[Vec3], color: ImColor32, thickness: f32) {
        let points = points
            .iter()
            .filter_map(|pos| world_to_screen(self.mvp, self.viewport, *pos))
            .collect::<Vec<_>>();

        if points.len() > 1 {
            self.draw_list
                .add_polyline(points, color)
                .thickness(thickness)
                .build();
        }
    }

    pub fn sector(
        &self,
        radius: f32,
        start_angle: f32,
        end_angle: f32,
        fill: ImColor32,
        thickness: f32,
    ) {
        let angle_delta = end_angle - start_angle;
        let step_count = steps(angle_delta.abs());
        let mut points = Vec::with_capacity(step_count);

        let step_size = angle_delta / (step_count - 1) as f32;

        points.push(Vec3::new(0.0, 0.0, 0.0));
        for step in (0..step_count).map(|i| step_size * i as f32) {
            // TODO optimize? cos sin only once before loop
            let x = f32::cos(start_angle + step) * radius;
            let z = f32::sin(start_angle + step) * radius;

            points.push(Vec3::new(x, 0.0, z));
        }

        let points = points
            .into_iter()
            .filter_map(|point| world_to_screen(self.mvp, self.viewport, point))
            .collect::<Vec<_>>();

        self.draw_list
            .add_polyline(points, fill)
            .thickness(thickness)
            .filled(true)
            .build();
    }
}
const STEPS_PER_RAD: f32 = 20.0;
fn steps(angle: f32) -> usize {
    (STEPS_PER_RAD * angle.abs()).ceil().max(1.0) as usize
}

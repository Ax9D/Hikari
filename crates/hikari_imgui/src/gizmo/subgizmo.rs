use hikari_math::{Transform, Vec3};
use imgui::{ImColor32, Ui};

use crate::{GizmoContext, GizmoState, GizmoStyle};

use super::{ray::Ray, rotate, scale, translate, Direction};

#[derive(Copy, Clone)]
pub(crate) struct SubGizmo {
    pub id: u32,
    pub state: GizmoState,
    pub style: GizmoStyle,
    pub direction: Direction,
    pub kind: SubGizmoKind,
    /// Whether this subgizmo is focused this frame
    pub focused: bool,
    /// Whether this subgizmo is active this frame
    pub active: bool,
}

impl SubGizmo {
    pub fn new(
        id: u32,
        state: GizmoState,
        style: GizmoStyle,
        direction: Direction,
        kind: SubGizmoKind,
    ) -> Self {
        Self {
            id,
            state,
            style,
            direction,
            kind,
            focused: false,
            active: false,
        }
    }

    pub fn local_normal(&self) -> Vec3 {
        match self.direction {
            Direction::X => Vec3::X,
            Direction::Y => Vec3::Y,
            Direction::Z => Vec3::Z,
            Direction::Screen => -self.state.view_forward(),
        }
    }

    pub fn normal(&self) -> Vec3 {
        let mut normal = self.local_normal();

        if self.state.local_space() && self.direction != Direction::Screen {
            normal = self.state.transform.rotation * normal;
        }

        normal
    }
    pub fn color(&self) -> ImColor32 {
        let mut color = self.style.color(self.direction);
        color.a = if self.focused {
            (self.style.focus_alpha * 255.0).round() as u8
        } else {
            (self.style.inactive_alpha * 255.0).round() as u8
        };

        color
    }
    pub fn pick(&self, context: &mut GizmoContext, ray: &Ray) -> Option<f32> {
        match self.kind {
            SubGizmoKind::TranslationVector => translate::pick_vector(self, context, ray),
            SubGizmoKind::TranslationPlane => translate::pick_plane(self, context, ray),
            SubGizmoKind::RotationAxis => rotate::pick(self, context, ray),
            SubGizmoKind::ScaleVector => scale::pick_vector(self, context, ray),
            SubGizmoKind::ScalePlane => scale::pick_plane(self, context, ray),
        }
    }

    /// Update this subgizmo based on pointer ray and interaction.
    pub fn update(&self, context: &mut GizmoContext, ray: &Ray) -> Option<Transform> {
        match self.kind {
            SubGizmoKind::TranslationVector => translate::update_vector(self, context, ray),
            SubGizmoKind::TranslationPlane => translate::update_plane(self, context, ray),
            SubGizmoKind::RotationAxis => rotate::update(self, context, ray),
            SubGizmoKind::ScaleVector => scale::update_vector(self, context, ray),
            SubGizmoKind::ScalePlane => scale::update_plane(self, context, ray),
        }
    }

    /// Draw this subgizmo
    pub fn draw(&self, context: &mut GizmoContext, ui: &Ui) {
        match self.kind {
            SubGizmoKind::TranslationVector => translate::draw_vector(self, ui),
            SubGizmoKind::TranslationPlane => translate::draw_plane(self, ui),
            SubGizmoKind::ScaleVector => scale::draw_vector(self, ui),
            SubGizmoKind::ScalePlane => scale::draw_plane(self, ui),
            SubGizmoKind::RotationAxis => rotate::draw(self, context, ui),
        }
    }
}

#[derive(Copy, Clone)]
pub(crate) enum SubGizmoKind {
    /// Rotation around an axis
    RotationAxis,
    /// Translation along a vector
    TranslationVector,
    /// Translation along a plane
    TranslationPlane,
    /// Scale along a vector
    ScaleVector,
    /// Scale along a plane
    ScalePlane,
}

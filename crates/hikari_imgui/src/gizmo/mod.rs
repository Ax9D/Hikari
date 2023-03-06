/// This is heavily based on https://github.com/urholaukkarinen/egui-gizmo
/// I have mostly adapted their code for use with imgui
/// All credit goes to Urho Laukkarinen <urho.laukkarinen@gmail.com>
mod draw;
mod math;
mod ray;
mod rotate;
mod scale;
mod subgizmo;
mod translate;

use std::ops::Sub;

use arrayvec::ArrayVec;
use hikari_math::{Mat4, Transform, Vec2, Vec3, Vec4, Vec4Swizzles};
use nohash_hasher::IntMap;
use subgizmo::SubGizmo;
use subgizmo::SubGizmoKind;

use self::ray::Ray;
use self::rotate::RotationState;
use self::scale::ScaleState;
use self::translate::TranslationState;

const TRANSLATE_X_ID: u32 = 0;
const TRANSLATE_Y_ID: u32 = 1;
const TRANSLATE_Z_ID: u32 = 2;
#[allow(dead_code)]
const TRANSLATE_SCREEN: u32 = 3;

const TRANSLATE_X_PLANE_ID: u32 = 4;
const TRANSLATE_Y_PLANE_ID: u32 = 5;
const TRANSLATE_Z_PLANE_ID: u32 = 6;

const SCALE_X_ID: u32 = 7;
const SCALE_Y_ID: u32 = 8;
const SCALE_Z_ID: u32 = 9;

const SCALE_X_PLANE_ID: u32 = 10;
const SCALE_Y_PLANE_ID: u32 = 11;
const SCALE_Z_PLANE_ID: u32 = 12;

const ROTATE_X_ID: u32 = 13;
const ROTATE_Y_ID: u32 = 14;
const ROTATE_Z_ID: u32 = 15;
const ROTATE_SCREEN_ID: u32 = 16;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Operation {
    Translate,
    Rotate,
    Scale,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Direction {
    X,
    Y,
    Z,
    Screen,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    Local,
    World,
}

#[derive(Default, Debug)]
pub struct GizmoContext {
    pub active_subgizmo_id: Option<u32>,
    translation_state: IntMap<u32, TranslationState>,
    scale_state: IntMap<u32, ScaleState>,
    rotation_state: IntMap<u32, RotationState>,
}
impl GizmoContext {
    pub fn new() -> GizmoContext {
        Self::default()
    }
    pub(crate) fn translation_state(&mut self, id: u32) -> &mut TranslationState {
        self.translation_state.entry(id).or_default()
    }
    pub(crate) fn scale_state(&mut self, id: u32) -> &mut ScaleState {
        self.scale_state.entry(id).or_default()
    }
    pub(crate) fn rotation_state(&mut self, id: u32) -> &mut RotationState {
        self.rotation_state.entry(id).or_default()
    }
    pub fn gizmo<'a, 'ui>(&'a mut self, ui: &'ui imgui::Ui) -> Gizmo<'a, 'ui> {
        Gizmo {
            context: self,
            ui,
            state: GizmoState::default(),
            style: GizmoStyle::default(),
            subgizmos: ArrayVec::new_const(),
            drag_last_frame: false,
        }
    }
}
#[derive(Clone, Copy)]
pub struct GizmoStyle {
    pub color_x: imgui::ImColor32,
    pub color_y: imgui::ImColor32,
    pub color_z: imgui::ImColor32,
    pub color_screen: imgui::ImColor32,

    pub inactive_alpha: f32,
    pub focus_alpha: f32,
    pub line_thickness: f32,
}
impl GizmoStyle {
    pub fn color(&self, direction: Direction) -> imgui::ImColor32 {
        match direction {
            Direction::X => self.color_x,
            Direction::Y => self.color_y,
            Direction::Z => self.color_z,
            Direction::Screen => self.color_screen,
        }
    }
}
impl Default for GizmoStyle {
    fn default() -> Self {
        Self {
            color_x: imgui::ImColor32::from_rgb(237, 69, 90),
            color_y: imgui::ImColor32::from_rgb(133, 204, 54),
            color_z: imgui::ImColor32::from_rgb(72, 143, 241),
            color_screen: imgui::ImColor32::WHITE,
            inactive_alpha: 0.723,
            focus_alpha: 1.0,
            line_thickness: 4.0,
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct Viewport {
    pub min: Vec2,
    pub max: Vec2,
}
impl Viewport {
    #[inline]
    pub fn unset() -> Self {
        Self {
            min: Vec2::new(-1.0, -1.0),
            max: Vec2::new(-1.0, -1.0),
        }
    }
    pub fn is_set(&self) -> bool {
        self != &Self::unset()
    }
    #[allow(dead_code)]
    #[inline]
    pub fn x(&self) -> f32 {
        self.min.x
    }
    #[allow(dead_code)]
    #[inline]
    pub fn y(&self) -> f32 {
        self.min.y
    }
    #[inline]
    pub fn width(&self) -> f32 {
        self.max.x - self.min.x
    }
    #[inline]
    pub fn height(&self) -> f32 {
        self.max.y - self.min.y
    }
    pub fn center(&self) -> Vec2 {
        (self.max + self.min) / 2.0
    }
}

#[derive(Clone, Copy)]
pub(crate) struct GizmoState {
    pub viewport: Viewport,
    pub cursor_pos: Vec2,
    pub operation: Operation,
    pub mode: Mode,
    pub view: Mat4,
    pub proj_view: Mat4,
    pub proj_view_inv: Mat4,
    pub model: Mat4,
    pub transform: Transform,
    pub mvp: Mat4,
    pub orthographic: bool,
    pub gizmo_size: f32,
    pub focus_distance: f32,
    pub scale_factor: f32,
    pub left_handed: bool,
}
impl Default for GizmoState {
    fn default() -> Self {
        Self {
            viewport: Viewport::unset(),
            cursor_pos: Vec2::ZERO,
            mode: Mode::World,
            operation: Operation::Translate,
            model: Mat4::IDENTITY,
            transform: Transform::default(),
            view: Mat4::IDENTITY,
            proj_view: Mat4::IDENTITY,
            proj_view_inv: Mat4::IDENTITY,
            mvp: Mat4::IDENTITY,
            orthographic: false,
            scale_factor: 0.0,
            gizmo_size: 75.0,
            focus_distance: 0.0,
            left_handed: false,
        }
    }
}
impl GizmoState {
    pub fn prepare(
        &mut self,
        ui: &imgui::Ui,
        transform: &Transform,
        projection: Mat4,
        view: Mat4,
        style: &GizmoStyle,
    ) {
        // If a viewport was not provided draw into the current window
        if !self.viewport.is_set() {
            let pos = ui.window_pos();
            let size = ui.window_size();
            self.viewport = Viewport {
                min: pos.into(),
                max: size.into(),
            };
        }
        self.cursor_pos = ui.io().mouse_pos.into();

        let model = transform.get_matrix();
        self.model = model;
        self.transform = transform.clone();
        self.view = view;
        self.proj_view = projection * view;
        self.proj_view_inv = self.proj_view.inverse();
        self.mvp = projection * view * model;

        self.scale_factor =
            self.mvp.as_ref()[15] / projection.as_ref()[0] / self.viewport.width() * 2.0;

        self.focus_distance = self.scale_factor * (style.line_thickness / 2.0 + 5.0);

        self.left_handed = if projection.z_axis.w == 0.0 {
            projection.z_axis.z > 0.0
        } else {
            projection.z_axis.w > 0.0
        };

    }
    /// Forward vector of the view camera
    pub fn view_forward(&self) -> Vec3 {
        self.view.row(2).xyz()
    }

    /// Right vector of the view camera
    pub fn view_right(&self) -> Vec3 {
        self.view.row(0).xyz()
    }

    /// Whether local mode is used
    pub fn local_space(&self) -> bool {
        self.mode == Mode::Local
    }
    pub fn pointer_ray(&self, ui: &imgui::Ui) -> Ray {
        let [hover_x, hover_y] = ui.io().mouse_pos;
        let viewport = self.viewport;

        let x = ((hover_x - viewport.min.x) / viewport.width()) * 2.0 - 1.0;
        let y = ((hover_y - viewport.min.y) / viewport.height()) * 2.0 - 1.0;

        let screen_to_world = self.proj_view_inv;
        let mut origin = screen_to_world * Vec4::new(x, -y, -1.0, 1.0);
        origin /= origin.w;

        let mut target = screen_to_world * Vec4::new(x, -y, 1.0, 1.0);

        // w is zero when far plane is set to infinity
        if target.w.abs() < 1e-7 {
            target.w = 1e-7;
        }

        target /= target.w;

        let direction = target.sub(origin).xyz().normalize();

        Ray {
            origin: origin.xyz(),
            direction,
        }
    }
}
const MAX_SUBGIZMOS: usize = 6;
pub struct Gizmo<'a, 'ui> {
    context: &'a mut GizmoContext,
    ui: &'ui imgui::Ui,
    state: GizmoState,
    style: GizmoStyle,
    subgizmos: ArrayVec<SubGizmo, MAX_SUBGIZMOS>,
    drag_last_frame: bool,
}

impl<'a, 'ui> Gizmo<'a, 'ui> {
    // /// Return true if mouse cursor is over any gizmo control (axis, plan or screen component)
    // pub fn is_over(&self) -> bool {todo!()}

    // /// Return true if mouse is_over or if the gizmo is in moving state
    // pub fn is_using(&self) -> bool {todo!()}

    /// Enable/disable the gizmo. Stay in the state until next call to Enable.
    /// gizmo is rendered with gray half transparent color when disabled
    // pub fn enable(mut self, enable: bool) -> Self {
    //     self.state.enable = enable;
    //     self
    // }

    pub fn viewport(mut self, min: Vec2, max: Vec2) -> Self {
        self.state.viewport = Viewport { min, max };
        self
    }
    pub fn mode(mut self, mode: Mode) -> Self {
        self.state.mode = mode;

        self
    }
    pub fn operation(mut self, operation: Operation) -> Self {
        self.state.operation = operation;

        self
    }
    pub fn orthographic(mut self, orthographic: bool) -> Self {
        self.state.orthographic = orthographic;

        self
    }
    pub fn size(mut self, size: f32) -> Self {
        self.state.gizmo_size = size;

        self
    }
    pub fn style(mut self, style: GizmoStyle) -> Self {
        self.style = style;

        self
    }
    fn new_translation(&self) -> [SubGizmo; 6] {
        [
            SubGizmo::new(
                TRANSLATE_X_ID,
                self.state,
                self.style,
                Direction::X,
                SubGizmoKind::TranslationVector,
            ),
            SubGizmo::new(
                TRANSLATE_Y_ID,
                self.state,
                self.style,
                Direction::Y,
                SubGizmoKind::TranslationVector,
            ),
            SubGizmo::new(
                TRANSLATE_Z_ID,
                self.state,
                self.style,
                Direction::Z,
                SubGizmoKind::TranslationVector,
            ),
            SubGizmo::new(
                TRANSLATE_X_PLANE_ID,
                self.state,
                self.style,
                Direction::X,
                SubGizmoKind::TranslationPlane,
            ),
            SubGizmo::new(
                TRANSLATE_Y_PLANE_ID,
                self.state,
                self.style,
                Direction::Y,
                SubGizmoKind::TranslationPlane,
            ),
            SubGizmo::new(
                TRANSLATE_Z_PLANE_ID,
                self.state,
                self.style,
                Direction::Z,
                SubGizmoKind::TranslationPlane,
            ),
        ]
    }
    fn new_scale(&self) -> [SubGizmo; 6] {
        [
            SubGizmo::new(
                SCALE_X_ID,
                self.state,
                self.style,
                Direction::X,
                SubGizmoKind::ScaleVector,
            ),
            SubGizmo::new(
                SCALE_Y_ID,
                self.state,
                self.style,
                Direction::Y,
                SubGizmoKind::ScaleVector,
            ),
            SubGizmo::new(
                SCALE_Z_ID,
                self.state,
                self.style,
                Direction::Z,
                SubGizmoKind::ScaleVector,
            ),
            SubGizmo::new(
                SCALE_X_PLANE_ID,
                self.state,
                self.style,
                Direction::X,
                SubGizmoKind::ScalePlane,
            ),
            SubGizmo::new(
                SCALE_Y_PLANE_ID,
                self.state,
                self.style,
                Direction::Y,
                SubGizmoKind::ScalePlane,
            ),
            SubGizmo::new(
                SCALE_Z_PLANE_ID,
                self.state,
                self.style,
                Direction::Z,
                SubGizmoKind::ScalePlane,
            ),
        ]
    }
    fn new_rotation(&self) -> [SubGizmo; 4] {
        [
            SubGizmo::new(
                ROTATE_X_ID,
                self.state,
                self.style,
                Direction::X,
                SubGizmoKind::RotationAxis,
            ),
            SubGizmo::new(
                ROTATE_Y_ID,
                self.state,
                self.style,
                Direction::Y,
                SubGizmoKind::RotationAxis,
            ),
            SubGizmo::new(
                ROTATE_Z_ID,
                self.state,
                self.style,
                Direction::Z,
                SubGizmoKind::RotationAxis,
            ),
            SubGizmo::new(
                ROTATE_SCREEN_ID,
                self.state,
                self.style,
                Direction::Screen,
                SubGizmoKind::RotationAxis,
            ),
        ]
    }
    fn pick_subgizmo(&mut self, ray: &Ray) -> Option<&mut SubGizmo> {
        self.subgizmos
            .iter_mut()
            .filter_map(|subgizmo| subgizmo.pick(self.context, ray).map(|t| (t, subgizmo)))
            .min_by(|(first, _), (second, _)| {
                first
                    .partial_cmp(second)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|(_, subgizmo)| subgizmo)
    }
    pub fn manipulate(
        mut self,
        transform: Transform,
        projection: Mat4,
        view: Mat4,
    ) -> Option<Transform> {
        hikari_dev::profile_function!();
        let ui = self.ui;

        self.state
            .prepare(ui, &transform, projection, view, &self.style);

        let mut out_transform = None;
        match self.state.operation {
            Operation::Translate => {
                for subgizmo in self.new_translation() {
                    self.subgizmos.push(subgizmo);
                }
            }
            Operation::Scale => {
                for subgizmo in self.new_scale() {
                    self.subgizmos.push(subgizmo);
                }
            }
            Operation::Rotate => {
                for subgizmo in self.new_rotation() {
                    self.subgizmos.push(subgizmo);
                }
            }
        };
        let dragging = ui.is_mouse_dragging(imgui::MouseButton::Left);
        let drag_started = !self.drag_last_frame && dragging;

        if ui.is_window_focused() {
            let ray = self.state.pointer_ray(ui);

            if self.context.active_subgizmo_id.is_none() {
                if let Some(subgizmo) = self.pick_subgizmo(&ray) {
                    subgizmo.focused = true;

                    if drag_started {
                        self.context.active_subgizmo_id = Some(subgizmo.id);
                    }
                }
            }
            let active_subgizmo = self
                .context
                .active_subgizmo_id
                .and_then(|id| self.subgizmos.iter_mut().find(|subgizmo| subgizmo.id == id));

            if let Some(subgizmo) = active_subgizmo {
                if dragging {
                    subgizmo.active = true;
                    subgizmo.focused = true;
                    out_transform = subgizmo.update(&mut self.context, &ray);

                    //println!("{:#?}", self.context);
                } else {
                    self.context.active_subgizmo_id = None;
                }
            }
        }

        for subgizmo in self.subgizmos.drain(..) {
            if self.context.active_subgizmo_id.is_none() || subgizmo.active {
                subgizmo.draw(self.context, ui);
            }
        }

        self.drag_last_frame = dragging;

        out_transform
    }
}

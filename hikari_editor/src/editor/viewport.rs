use crate::imgui;
use crate::imgui::gizmo::*;
use hikari::asset::AssetManager;
use hikari::core::{Entity, Time};
use hikari::g3d::{Light, LightKind};
use hikari::math::*;
use hikari::{
    core::World,
    g3d::{Camera, ShaderLibrary},
    math::{Transform, Vec2},
    pbr::WorldRenderer,
    render::imgui_support::TextureExt,
};
use hikari_editor::*;

use super::camera::CameraState;
use super::meta::EditorOnly;
use super::{camera, icons, Editor};

struct GizmoState {
    context: GizmoContext,
    operation: Option<Operation>,
    mode: Mode,
}
impl Default for GizmoState {
    fn default() -> Self {
        Self {
            context: Default::default(),
            operation: Some(Operation::Translate),
            mode: Mode::World,
        }
    }
}
#[derive(Default)]
pub struct Viewport {
    gizmo_state: GizmoState,
    camera_state: CameraState,
}
fn gizmo_toolbar(ui: &imgui::Ui, state: &mut GizmoState, editor_camera: &mut Camera) {
    let parent_pos = ui.window_pos();
    let parent_size = ui.window_size();
    let size = [200.0, 50.0];
    let pos_offset = [15.0, -15.0];
    let pos = [
        parent_pos[0] + parent_size[0] - pos_offset[0] - size[0],
        parent_pos[1] + pos_offset[1] + size[1],
    ];

    ui.window("Gizmo Toolbar")
        .position(pos, imgui::Condition::Always)
        .size(size, imgui::Condition::Always)
        .resizable(false)
        .flags(
            imgui::WindowFlags::NO_TITLE_BAR
        | imgui::WindowFlags::NO_RESIZE
        | imgui::WindowFlags::NO_SCROLLBAR
        //imgui:: | WindowFlags::NO_INPUTS
        | imgui::WindowFlags::NO_SAVED_SETTINGS
        | imgui::WindowFlags::NO_DOCKING
        | imgui::WindowFlags::NO_DECORATION
        | imgui::WindowFlags::NO_BACKGROUND, //| imgui::WindowFlags::NO_FOCUS_ON_APPEARING
                                                         //| imgui::WindowFlags::NO_BRING_TO_FRONT_ON_FOCUS
        )
        .build(|| {
            fn draw_operation(
                ui: &imgui::Ui,
                operation: Operation,
                icon: &str,
                current: &mut Option<Operation>,
            ) {
                let mut _style_vars = None;
                if let Some(current) = *current {
                    if current == operation {
                        let token1 = ui.push_style_color(
                            imgui::StyleColor::Button,
                            imgui::ImColor32::from_rgb(0, 115, 207).to_rgba_f32s(),
                        );
                        let token2 = ui.push_style_color(
                            imgui::StyleColor::ButtonHovered,
                            imgui::ImColor32::from_rgb(1, 151, 246).to_rgba_f32s(),
                        );
                        _style_vars = Some((token1, token2));
                    }
                }

                if ui.button(icon) {
                    *current = Some(operation);
                }
            }
            let _style_token = ui.push_style_var(imgui::StyleVar::ItemSpacing([2.0, 0.0]));
            let _style_token = ui.push_style_var(imgui::StyleVar::FrameRounding(5.0));
            if ui.button(icons::MOUSE_SELECT) {
                state.operation = None;
            }
            ui.same_line();

            draw_operation(
                ui,
                Operation::Translate,
                icons::GIZMO_TRANSLATE,
                &mut state.operation,
            );
            ui.same_line();

            draw_operation(
                ui,
                Operation::Rotate,
                icons::GIZMO_ROTATE,
                &mut state.operation,
            );
            ui.same_line();

            draw_operation(
                ui,
                Operation::Scale,
                icons::GIZMO_SCALE,
                &mut state.operation,
            );
            ui.same_line();

            let clicked = match state.mode {
                Mode::Local => ui.button(icons::GIZMO_LOCAL),
                Mode::World => ui.button(icons::GIZMO_WORLD),
            };

            if clicked {
                state.mode = match state.mode {
                    Mode::Local => Mode::World,
                    Mode::World => Mode::Local,
                };
            }
            ui.same_line();

            if ui.button("C") {
                ui.open_popup("Editor Camera Settings");
            }

            ui.popup("Editor Camera Settings", || {
                imgui::Drag::new("near").build(ui, &mut editor_camera.near);
                imgui::Drag::new("far").build(ui, &mut editor_camera.far);

                match &mut editor_camera.projection {
                    hikari::g3d::Projection::Perspective(fov) => {
                        imgui::Drag::new("fov").build(ui, fov);
                    }
                    hikari::g3d::Projection::Orthographic => todo!(),
                }

                imgui::Drag::new("exposure").build(ui, &mut editor_camera.exposure);
            });
        });
}
fn draw_dir_light(ui: &imgui::Ui, world: &mut World, viewport_min: Vec2, viewport_max: Vec2) {
    #[derive(Clone, Copy)]
    pub(crate) struct Viewport {
        min: Vec2,
        max: Vec2,
    }
    impl Viewport {
        fn center(&self) -> Vec2 {
            (self.min + self.max) / 2.0
        }
        fn width(&self) -> f32 {
            (self.max - self.min).x
        }
        fn height(&self) -> f32 {
            (self.max - self.min).y
        }
    }
    pub(crate) fn world_to_screen(mvp: Mat4, viewport: Viewport, pos: Vec3) -> Option<Vec2> {
        let mut pos = mvp * Vec4::from((pos, 1.0));

        if pos.w < 0.0 {
            return None;
        }

        pos /= pos.w;
        pos.y *= -1.0;

        let center = viewport.center();
        Some(Vec2::new(
            center.x + pos.x * viewport.width() / 2.0,
            center.y + pos.y * viewport.height() / 2.0,
        ))
    }

    let camera_entity = get_editor_camera(world);
    if let Some((_, (transform, _))) = world
        .query::<(&Transform, &Light)>()
        .iter()
        .filter(|(_, (_, light))| matches!(light.kind, LightKind::Directional))
        .next()
    {
        let light_dir = transform.forward();
        let camera = world.get_component::<&Camera>(camera_entity).unwrap();
        let camera_transform = world.get_component::<&Transform>(camera_entity).unwrap();

        let viewport = Viewport {
            min: viewport_min,
            max: viewport_max,
        };

        let view_proj = camera.get_projection_matrix(viewport.width(), viewport.height())
            * camera_transform.get_matrix().inverse();

        let mvp = view_proj * Mat4::from_translation(transform.position);
        let start_2d = world_to_screen(mvp, viewport, Vec3::ZERO);
        let end_2d = world_to_screen(mvp, viewport, Vec3::ZERO + light_dir * 5.0);

        if let Some((start, end)) = start_2d.zip(end_2d) {
            ui.get_window_draw_list()
                .add_line(start, end, imgui::ImColor32::WHITE)
                .thickness(1.0)
                .build();
        }
    }
}
fn get_editor_camera(world: &mut World) -> Entity {
    let create_camera;

    if let Some((entity, _)) = world.query::<(&EditorOnly, &mut Camera)>().iter().next() {
        return entity;
    } else {
        create_camera = true;
    }

    if create_camera {
        let camera_entity =
            world.create_entity_with((Transform::default(), EditorOnly, Camera::default()));
        return camera_entity;
    } else {
        unreachable!()
    }
}
pub fn draw(ui: &imgui::Ui, editor: &mut Editor, state: EngineState) -> anyhow::Result<()> {
    let viewport = &mut editor.viewport;
    let outliner = &mut editor.outliner;

    let mut renderer = state.get_mut::<WorldRenderer>().unwrap();
    let mut world = state.get_mut::<World>().unwrap();
    let shader_lib = state.get_mut::<ShaderLibrary>().unwrap();
    let asset_manager = state.get::<AssetManager>().unwrap();

    let dt = state.get::<Time>().unwrap().dt();

    //ui.set_keyboard_focus_here();
    ui.window("Viewport")
        .size([950.0, 200.0], imgui::Condition::Once)
        .resizable(true)
        .build(|| {
            let window_size_float = ui.content_region_avail();

            let window_size = (window_size_float[0], window_size_float[1]);
            let renderer_size = renderer.viewport();

            if window_size != renderer_size {
                renderer.set_viewport(window_size.0, window_size.1);
            }

            let editor_camera = get_editor_camera(&mut world);

            let viewport_min = Vec2::new(
                ui.window_pos()[0] + ui.window_content_region_min()[0],
                ui.window_pos()[1] + ui.window_content_region_min()[1],
            );

            let viewport_max = Vec2::new(
                viewport_min[0] + window_size_float[0],
                viewport_min[1] + window_size_float[1],
            );

            if ui.is_window_focused() {
                camera::manipulate(
                    ui,
                    &mut viewport.camera_state,
                    &mut world
                        .get_component::<&mut Transform>(editor_camera)
                        .unwrap(),
                    dt,
                );

                ui.get_window_draw_list()
                    .add_rect(viewport_min, viewport_max, imgui::ImColor32::WHITE)
                    .thickness(0.5)
                    .build();
            }

            let pbr_output = renderer
                .render_editor(&world, Some(editor_camera), &shader_lib, &asset_manager)
                .expect("Failed to render editor viewport");

            let pbr_output = ui.get_texture_id(pbr_output);
            imgui::Image::new(pbr_output, window_size_float).build(ui);

            {
                draw_dir_light(ui, &mut world, viewport_min, viewport_max);
                let mut editor_camera = world.get_component::<&mut Camera>(editor_camera).unwrap();
                gizmo_toolbar(ui, &mut viewport.gizmo_state, &mut editor_camera);
            }

            if let Some(entity) = outliner.selected {
                if let Ok(mut query) = world.query_one::<(&Camera, &mut Transform)>(editor_camera) {
                    let (camera, cam_transform) = query.get().unwrap();

                    if let Ok(mut transform) = world.get_component::<&mut Transform>(entity) {
                        let projection = camera.get_projection_matrix(window_size.0, window_size.1);
                        let view = cam_transform.get_matrix().inverse();

                        if let Some(operation) = viewport.gizmo_state.operation {
                            // If transform changed update it
                            if let Some(changed_transform) = viewport
                                .gizmo_state
                                .context
                                .gizmo(ui)
                                .operation(operation)
                                .mode(viewport.gizmo_state.mode)
                                .viewport(viewport_min, viewport_max)
                                .manipulate(*transform, projection, view)
                            {
                                *transform = changed_transform;
                            }
                        }
                    }
                }
            }
        });

    Ok(())
}

use hikari_math::Transform;
use itertools::izip;
use simple_logger::SimpleLogger;
use std::{collections::HashMap, sync::Arc};
use winit::{
    dpi::LogicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

pub use hikari_3d::texture::*;
use hikari_imgui as imgui;
use hikari_render as rg;
mod common;

const WIDTH: u32 = 1920;
const HEIGHT: u32 = 1080;

#[repr(C)]
#[derive(Copy, Clone)]
struct Vertex {
    position: [f32; 3],
    normal: [f32; 3],
    tc0: [f32; 2],
    tc1: [f32; 2],
}

struct Material {
    albedo: Option<Arc<Texture2D>>,
    albedo_set: i32,
    albedo_factor: hikari_math::Vec4,
    roughness: Option<Arc<Texture2D>>,
    roughness_set: i32,
    roughness_factor: f32,
    metallic: Option<Arc<Texture2D>>,
    metallic_set: i32,
    metallic_factor: f32,
    normal: Option<Arc<Texture2D>>,
    normal_set: i32,
}
struct Mesh {
    vertices: rg::GpuBuffer<Vertex>,
    indices: rg::GpuBuffer<u32>,
    material: Arc<Material>,
}
struct Model {
    meshes: Vec<Mesh>,
}
struct GameObject {
    model: Arc<Model>,
    transform: Transform,
}

impl GameObject {
    pub fn new(model: &Arc<Model>) -> Self {
        let transform = Transform::default();

        Self {
            model: model.clone(),
            transform,
        }
    }
}
struct Settings {
    fxaa: bool,
    vsync: bool,
    width: u32,
    height: u32,
}
type Args = (Scene, Settings, imgui::DrawData);
struct Camera {
    transform: Transform,
    inner: hikari_3d::Camera,
}
impl Camera {
    pub fn get_projection_matrix(&self, width: f32, height: f32) -> hikari_math::Mat4 {
        self.inner.get_projection_matrix(width, height)
    }
    pub fn get_view_matrix(&self) -> hikari_math::Mat4 {
        //hikari_math::Mat4::look_at_rh()
        hikari_math::Mat4::from_rotation_translation(
            self.transform.rotation,
            self.transform.position,
        )
        .inverse()
    }
}

struct Light {
    light: hikari_3d::Light,
    transform: Transform,
}
struct Scene {
    objects: Vec<GameObject>,
    camera: Camera,
    dir_light: Light,
}
struct UiState {
    gizmo: imgui::gizmo::GizmoContext,
    gizmo_operation: imgui::gizmo::Operation,
    gizmo_mode: imgui::gizmo::Mode,

    euler_cache: HashMap<usize, (f32, f32, f32)>, // Object Index To Euler Angles in Editor
}
fn load_mesh(
    device: &Arc<rg::Device>,
    path: &str,
) -> Result<Vec<Model>, Box<dyn std::error::Error>> {
    let scene = hikari_3d::old::Scene::load(path)?;
    let mut textures = Vec::new();
    for texture in &scene.textures {
        let data = &texture.data;

        // let mut path = String::from("/home/atri/test_img/");
        // path.push_str(texture.name());
        // path.push_str(".png");
        // save_image(&path, data, texture.width(), texture.height());

        let config = TextureConfig {
            format: texture.format,
            filtering: FilterMode::Linear,
            wrap_x: texture.wrap_x,
            wrap_y: texture.wrap_y,
            aniso_level: 16.0,
            generate_mips: texture.generate_mips,
        };

        let texture = Texture2D::new(device, data, texture.width, texture.height, config)?;

        textures.push(Arc::new(texture));
    }

    let mut materials = Vec::new();

    for material in &scene.materials {
        let albedo = material
            .albedo_map
            .as_ref()
            .map(|desc| textures[desc.index].clone());
        let albedo_factor = material.albedo;

        let metallic = material
            .metallic_map
            .as_ref()
            .map(|desc| textures[desc.index].clone());

        let metallic_factor = material.metallic;

        let roughness = material
            .roughness_map
            .as_ref()
            .map(|desc| textures[desc.index].clone());
        let roughness_factor = material.roughness;

        let normal = material
            .normal_map
            .as_ref()
            .map(|desc| textures[desc.index].clone());

        let albedo_set = material
            .albedo_map
            .as_ref()
            .map(|desc| desc.tex_coord_set as i32)
            .unwrap_or(-1);
        let roughness_set = material
            .roughness_map
            .as_ref()
            .map(|desc| desc.tex_coord_set as i32)
            .unwrap_or(-1);
        let metallic_set = material
            .metallic_map
            .as_ref()
            .map(|desc| desc.tex_coord_set as i32)
            .unwrap_or(-1);
        let normal_set = material
            .normal_map
            .as_ref()
            .map(|desc| desc.tex_coord_set as i32)
            .unwrap_or(-1);

        materials.push(Arc::new(Material {
            albedo,
            albedo_set,
            albedo_factor,
            roughness,
            roughness_set,
            roughness_factor,
            metallic,
            metallic_set,
            metallic_factor,
            normal,
            normal_set,
        }));
    }
    let mut models = Vec::new();
    for model in &scene.models {
        let mut meshes = Vec::new();
        for mesh in &model.meshes {
            let mut vertex_data = Vec::new();
            for (&position, &normal, &tc0, &tc1) in izip!(
                mesh.positions.iter(),
                mesh.normals.iter(),
                mesh.texcoord0.iter(),
                mesh.texcoord1.iter()
            ) {
                vertex_data.push(Vertex {
                    position: position.into(),
                    normal: normal.into(),
                    tc0: tc0.into(),
                    tc1: tc1.into(),
                });
            }

            let mut vertices = rg::create_vertex_buffer(device, vertex_data.len())?;
            vertices.upload(&vertex_data, 0)?;

            let mut indices = rg::create_index_buffer(device, mesh.indices.len())?;
            indices.upload(&mesh.indices, 0)?;

            meshes.push(Mesh {
                vertices,
                material: materials[mesh.material.unwrap()].clone(),
                indices,
            })
        }
        models.push(Model { meshes });
    }

    Ok(models)
}
fn depth_prepass(
    device: &Arc<rg::Device>,
    gb: &mut rg::GraphBuilder<Args>,
) -> rg::GpuHandle<rg::SampledImage> {
    let shader = rg::ShaderProgramBuilder::vertex_and_fragment(
        "DepthPrepass",
        &rg::ShaderCode {
            entry_point: "main",
            data: rg::ShaderData::Glsl(
                std::fs::read_to_string("examples/shaders/depth_only.vert").unwrap(),
            ),
        },
        &rg::ShaderCode {
            entry_point: "main",
            data: rg::ShaderData::Glsl(
                std::fs::read_to_string("examples/shaders/empty.frag").unwrap(),
            ),
        },
    )
    .build(device)
    .expect("Failed to create shader");

    let depth_output = gb
        .create_image(
            "PrepassDepth",
            rg::ImageConfig::depth_only(device),
            rg::ImageSize::default_xy(),
        )
        .expect("Failed to create depth image");

    #[repr(C)]
    #[derive(Copy, Clone)]
    struct UBO {
        view_proj: [f32; 16],
    }

    let mut ubo = rg::PerFrame::new([
        rg::create_uniform_buffer::<UBO>(device, 1).unwrap(),
        rg::create_uniform_buffer::<UBO>(device, 1).unwrap(),
    ]);

    #[repr(C)]
    #[derive(Copy, Clone)]
    struct PushConstants {
        transform: hikari_math::Mat4,
    }
    let layout = rg::VertexInputLayout::builder()
        .buffer(
            &[
                rg::ShaderDataType::Vec3f,
                rg::ShaderDataType::Vec3f,
                rg::ShaderDataType::Vec2f,
                rg::ShaderDataType::Vec2f,
            ],
            rg::StepMode::Vertex,
        )
        .build();
    gb.add_renderpass(
        rg::Renderpass::<Args>::new(
            "DepthPrepass",
            rg::ImageSize::default_xy(),
            move |cmd, (scene, settings, _)| {
                cmd.set_shader(&shader);
                cmd.set_vertex_input_layout(layout);

                cmd.set_depth_stencil_state(rg::DepthStencilState {
                    depth_test_enabled: true,
                    depth_write_enabled: true,
                    depth_compare_op: rg::CompareOp::Less,
                    ..Default::default()
                });

                let proj = scene
                    .camera
                    .get_projection_matrix(settings.width as f32, settings.height as f32);
                let view = scene.camera.get_view_matrix();
                let view_proj = (proj * view).to_cols_array();

                ubo.get_mut().mapped_slice_mut()[0] = UBO { view_proj };

                cmd.set_uniform_buffer(ubo.get(), 0..1, 0, 0);

                for object in &scene.objects {
                    let transform = object.transform.get_matrix();
                    let model = &object.model;
                    for mesh in &model.meshes {
                        {
                            hikari_dev::profile_scope!("Set vertex and index buffers");
                            cmd.set_vertex_buffer(&mesh.vertices, 0);
                            cmd.set_index_buffer(&mesh.indices);
                        }

                        cmd.push_constants(&PushConstants { transform }, 0);

                        // println!(
                        //     "{:?} {:?} {:?} {:?}",
                        //     albedo.raw().image(),
                        //     roughness.raw().image(),
                        //     metallic.raw().image(),
                        //     normal.raw().image()
                        // );

                        cmd.draw_indexed(0..mesh.indices.capacity(), 0, 0..1);
                    }
                }

                ubo.next_frame();
            },
        )
        .draw_image(&depth_output, rg::AttachmentConfig::depth_only_default()),
    );

    depth_output
}
fn pbr_pass(
    device: &Arc<rg::Device>,
    depth_prepass: &rg::GpuHandle<rg::SampledImage>,
    gb: &mut rg::GraphBuilder<Args>,
) -> rg::GpuHandle<rg::SampledImage> {
    #[repr(C)]
    #[derive(Debug, Copy, Clone)]
    struct Material {
        albedo: hikari_math::Vec4,
        roughness: f32,
        metallic: f32,
        albedo_uv_set: i32,
        roughness_uv_set: i32,
        metallic_uv_set: i32,
        normal_uv_set: i32,
    }

    #[repr(C)]
    #[derive(Debug, Copy, Clone)]
    struct PushConstants {
        transform: hikari_math::Mat4,
        material: Material,
    }

    #[repr(C)]
    #[derive(Copy, Clone, Default)]
    struct DirLight {
        intensity: f32,
        color: hikari_math::Vec3A,
        direction: hikari_math::Vec3A,
    }
    #[repr(C)]
    #[derive(Copy, Clone, Default)]

    struct UBO {
        camera_position: hikari_math::Vec3A,
        view_proj: [f32; 16],
        exposure: f32,

        dir_light: DirLight,
    }

    log::debug!("sizeof(UBO)={}", std::mem::size_of::<UBO>());

    let shader = rg::ShaderProgramBuilder::vertex_and_fragment(
        "PBR",
        &rg::ShaderCode {
            entry_point: "main",
            data: rg::ShaderData::Glsl(
                std::fs::read_to_string("examples/shaders/pbr.vert").unwrap(),
            ),
        },
        &rg::ShaderCode {
            entry_point: "main",
            data: rg::ShaderData::Glsl(
                std::fs::read_to_string("examples/shaders/pbr.frag").unwrap(),
            ),
        },
    )
    .build(device)
    .expect("Failed to create shader");

    let (checkerboard, width, height) =
        hikari_3d::old::image::load_from_file("examples/checkerboard.png")
            .expect("Failed to load checkerboard texture");

    let checkerboard = Texture2D::new(
        device,
        &checkerboard,
        width,
        height,
        TextureConfig {
            format: Format::RGBA8,
            wrap_x: WrapMode::Repeat,
            wrap_y: WrapMode::Repeat,
            filtering: FilterMode::Linear,
            aniso_level: 9.0,
            generate_mips: true,
        },
    )
    .expect("Failed to create checkerboard texture");

    let checkerboard = Arc::new(checkerboard);

    let black = Texture2D::new(
        device,
        &[0, 0, 0, 255],
        1,
        1,
        TextureConfig {
            format: Format::RGBA8,
            wrap_x: WrapMode::Repeat,
            wrap_y: WrapMode::Repeat,
            filtering: FilterMode::Linear,
            aniso_level: 0.0,
            generate_mips: false,
        },
    )
    .expect("Failed to create black texture");

    let black = Arc::new(black);

    let mut ubo = rg::PerFrame::new([
        rg::create_uniform_buffer::<UBO>(device, 1).unwrap(),
        rg::create_uniform_buffer::<UBO>(device, 1).unwrap(),
    ]);
    let layout = rg::VertexInputLayout::builder()
        .buffer(
            &[
                rg::ShaderDataType::Vec3f,
                rg::ShaderDataType::Vec3f,
                rg::ShaderDataType::Vec2f,
                rg::ShaderDataType::Vec2f,
            ],
            rg::StepMode::Vertex,
        )
        .build();

    let color_output = gb
        .create_image(
            "PBRColor",
            rg::ImageConfig::color2d(),
            rg::ImageSize::default_xy(),
        )
        .expect("Failed to create PBR attachments");
    // let depth_output = gb
    //     .create_image(
    //         "PBRDepth",
    //         rg::ImageConfig::depth_stencil(device),
    //         rg::ImageSize::default(),
    //     )
    //     .expect("Failed to create PBR attachments");
    gb.add_renderpass(
        rg::Renderpass::<Args>::new(
            "PBR",
            rg::ImageSize::default_xy(),
            move |cmd, (scene, settings, _)| {
                let proj = scene
                    .camera
                    .get_projection_matrix(settings.width as f32, settings.height as f32);
                let view = scene.camera.get_view_matrix();

                let view_proj = (proj * view).to_cols_array();

                use hikari_math::Vec3;
                let direction = scene.dir_light.transform.rotation * -Vec3::Y;

                ubo.get_mut().mapped_slice_mut()[0] = UBO {
                    view_proj,
                    camera_position: scene.camera.transform.position.into(),
                    exposure: scene.camera.inner.exposure,
                    dir_light: DirLight {
                        color: scene.dir_light.light.color.into(),
                        direction: direction.into(),
                        intensity: scene.dir_light.light.intensity,
                    },
                };

                cmd.set_shader(&shader);

                cmd.set_vertex_input_layout(layout);

                cmd.set_depth_stencil_state(rg::DepthStencilState {
                    depth_test_enabled: true,
                    depth_write_enabled: false,
                    depth_compare_op: rg::CompareOp::Equal,
                    ..Default::default()
                });

                cmd.set_uniform_buffer(ubo.get(), 0..1, 0, 0);

                {
                    hikari_dev::profile_scope!("Render scene");
                    for object in &scene.objects {
                        let transform = object.transform.get_matrix();
                        let model = &object.model;
                        for mesh in &model.meshes {
                            {
                                hikari_dev::profile_scope!("Set vertex and index buffers");
                                cmd.set_vertex_buffer(&mesh.vertices, 0);
                                cmd.set_index_buffer(&mesh.indices);
                            }
                            let material = Material {
                                albedo: mesh.material.albedo_factor,
                                roughness: mesh.material.roughness_factor,
                                metallic: mesh.material.metallic_factor,
                                albedo_uv_set: mesh.material.albedo_set,
                                roughness_uv_set: mesh.material.roughness_set,
                                metallic_uv_set: mesh.material.metallic_set,
                                normal_uv_set: mesh.material.normal_set,
                            };

                            let pc = PushConstants {
                                transform,
                                material,
                            };

                            cmd.push_constants(&pc, 0);

                            let albedo = mesh.material.albedo.as_ref().unwrap_or(&checkerboard);
                            let roughness = mesh.material.roughness.as_ref().unwrap_or(&black);
                            let metallic = mesh.material.metallic.as_ref().unwrap_or(&black);
                            let normal = mesh.material.normal.as_ref().unwrap_or(&black);

                            // println!(
                            //     "{:?} {:?} {:?} {:?}",
                            //     albedo.raw().image(),
                            //     roughness.raw().image(),
                            //     metallic.raw().image(),
                            //     normal.raw().image()
                            // );
                            cmd.set_image(albedo.raw(), 1, 0);
                            cmd.set_image(roughness.raw(), 1, 1);
                            cmd.set_image(metallic.raw(), 1, 2);
                            cmd.set_image(normal.raw(), 1, 3);

                            cmd.draw_indexed(0..mesh.indices.capacity(), 0, 0..1);
                        }
                    }
                }
                ubo.next_frame();
            },
        )
        .draw_image(&color_output, rg::AttachmentConfig::color_default(0))
        .draw_image(
            &depth_prepass,
            rg::AttachmentConfig {
                kind: rg::AttachmentKind::DepthOnly,
                access: rg::AccessType::DepthStencilAttachmentRead,
                load_op: rg::vk::AttachmentLoadOp::LOAD,
                store_op: rg::vk::AttachmentStoreOp::STORE,
                stencil_load_op: rg::vk::AttachmentLoadOp::DONT_CARE,
                stencil_store_op: rg::vk::AttachmentStoreOp::DONT_CARE,
            },
        ),
    );

    color_output
}
fn fxaa_pass(
    device: &Arc<rg::Device>,
    pbr_pass: rg::GpuHandle<rg::SampledImage>,
    gb: &mut rg::GraphBuilder<Args>,
) -> rg::GpuHandle<rg::SampledImage> {
    let vertex = r"
    #version 450
    vec2 positions[6] = vec2[](
        vec2(-1, -1),
        vec2(1, -1),
        vec2(1, 1),
        vec2(-1, 1),
        vec2(-1, -1),
        vec2(1, 1)
    );
    vec2 texCoords[6] = vec2[](
        vec2(0, 1),
        vec2(1, 1),
        vec2(1, 0),
        vec2(0, 0),
        vec2(0, 1),
        vec2(1, 0)
    );
    layout(location = 0) out vec2 texCoord;
    void main() {
        gl_Position = vec4(positions[gl_VertexIndex], 0.0, 1.0);
        texCoord = texCoords[gl_VertexIndex];
    }
    ";

    let shader = rg::ShaderProgramBuilder::vertex_and_fragment(
        "FXAA",
        &rg::ShaderCode {
            entry_point: "main",
            data: rg::ShaderData::Glsl(vertex.to_string()),
        },
        &rg::ShaderCode {
            entry_point: "main",
            data: rg::ShaderData::Glsl(
                std::fs::read_to_string("examples/shaders/fxaa.frag").unwrap(),
            ),
        },
    )
    .build(device)
    .expect("Failed to create shader");

    #[repr(C)]
    #[derive(Copy, Clone)]
    struct PushConstants {
        res: hikari_math::Vec2,
        enabled: i32,
    }
    let output = gb
        .create_image(
            "fxaa_output",
            rg::ImageConfig::color2d(),
            rg::ImageSize::default_xy(),
        )
        .expect("Failed to create fxaa output");
    gb.add_renderpass(
        rg::Renderpass::<Args>::new(
            "FXAA",
            rg::ImageSize::default_xy(),
            move |cmd, (_, settings, _)| {
                cmd.set_shader(&shader);

                cmd.push_constants(
                    &PushConstants {
                        res: hikari_math::vec2(settings.width as f32, settings.height as f32),
                        enabled: settings.fxaa as _,
                    },
                    0,
                );

                cmd.draw(0..6, 0..1);
            },
        )
        .draw_image(&output, rg::AttachmentConfig::color_default(0))
        .sample_image(
            &pbr_pass,
            rg::AccessType::FragmentShaderReadSampledImageOrUniformTexelBuffer,
            0,
        ),
    );

    output
}
fn imgui_pass(
    device: &Arc<rg::Device>,
    imgui: &mut rg::imgui_support::Backend,
    gb: &mut rg::GraphBuilder<Args>,
) -> rg::GpuHandle<rg::SampledImage> {
    let mut renderer = rg::imgui_support::Renderer::new(
        device,
        imgui,
        rg::vk::Format::R8G8B8A8_UNORM,
        device.supported_depth_stencil_format(),
        false,
    )
    .expect("Failed to create imgui renderer");

    let imgui_image = gb
        .create_image(
            "imgui_output",
            rg::ImageConfig::color2d(),
            rg::ImageSize::default_xy(),
        )
        .expect("Failed to create imgui image");
    gb.add_renderpass(
        rg::Renderpass::<Args>::new(
            "ImguiRenderer",
            rg::ImageSize::default_xy(),
            move |cmd, (_, _, draw_data)| {
                renderer
                    .render(cmd.raw(), draw_data)
                    .expect("Failed to render imgui");
            },
        )
        .draw_image(&imgui_image, rg::AttachmentConfig::color_default(0)),
    );

    imgui_image
}
fn rotation_alt(
    ui: &imgui::Ui,
    quat: &mut hikari_math::Quat,
    entity_id: usize,
    euler_cache: &mut HashMap<usize, (f32, f32, f32)>,
) {
    let (x, y, z) = euler_cache
        .entry(entity_id)
        .or_insert_with(|| quat.to_euler(hikari_math::EulerRot::XYZ))
        .clone();

    let externally_changed = !hikari_math::Quat::from_euler(hikari_math::EulerRot::XYZ, x, y, z)
        .abs_diff_eq(*quat, std::f32::EPSILON);

    let (x, y, z) = if externally_changed {
        quat.to_euler(hikari_math::EulerRot::XYZ)
    } else {
        (x, y, z)
    };

    let mut angles = [x.to_degrees(), y.to_degrees(), z.to_degrees()];

    let changed = imgui::Drag::new("Rotation")
        .speed(0.5)
        .build_array(ui, &mut angles);

    if changed {
        let (x, y, z) = (
            angles[0].to_radians(),
            angles[1].to_radians(),
            angles[2].to_radians(),
        );
        *euler_cache.get_mut(&entity_id).unwrap() = (x, y, z);
        *quat = hikari_math::Quat::from_euler(hikari_math::EulerRot::XYZ, x, y, z);
    }
}
fn transform_controls(
    ui: &imgui::Ui,
    transform: &mut Transform,
    entity_id: usize,
    state: &mut UiState,
) {
    let mut position: [f32; 3] = transform.position.into();
    imgui::Drag::new("position")
        .speed(0.1)
        .build_array(ui, &mut position);
    transform.position = position.into();

    rotation_alt(
        ui,
        &mut transform.rotation,
        entity_id,
        &mut state.euler_cache,
    );

    let mut scale: [f32; 3] = transform.scale.into();
    imgui::Drag::new("scale")
        .speed(0.2)
        .build_array(ui, &mut scale);
    transform.scale = hikari_math::Vec3::from(scale);
}
fn imgui_update(
    ui: &imgui::Ui,
    scene: &mut Scene,
    ui_state: &mut UiState,
    settings: &mut Settings,
) {
    //ui.show_demo_window(&mut true);
    ui.window("Transform Object").build(|| {
        transform_controls(ui, &mut scene.objects[1].transform, 1, ui_state);

        let operations = [
            imgui::gizmo::Operation::Translate,
            imgui::gizmo::Operation::Rotate,
            imgui::gizmo::Operation::Scale,
        ];
        let mut index = operations
            .iter()
            .position(|operation| operation == &ui_state.gizmo_operation)
            .unwrap();
        ui.combo("Gizmo Operation", &mut index, &operations, |item| {
            std::borrow::Cow::Owned(format!("{:?}", item))
        });
        ui_state.gizmo_operation = operations[index];

        let modes = [imgui::gizmo::Mode::World, imgui::gizmo::Mode::Local];
        let mut index = modes
            .iter()
            .position(|mode| mode == &ui_state.gizmo_mode)
            .unwrap();
        ui.combo("Gizmo Mode", &mut index, &modes, |item| {
            std::borrow::Cow::Owned(format!("{:?}", item))
        });

        ui_state.gizmo_mode = modes[index];

        let _nobg = ui.push_style_color(
            imgui::StyleColor::WindowBg,
            imgui::ImColor32::TRANSPARENT.to_rgba_f32s(),
        );
        let _noborder = ui.push_style_color(
            imgui::StyleColor::Border,
            imgui::ImColor32::TRANSPARENT.to_rgba_f32s(),
        );
        let _norounding = ui.push_style_var(imgui::StyleVar::WindowRounding(0.0));

        use imgui::WindowFlags;
        ui.window("Gizmo")
            .position([0.0, 0.0], imgui::Condition::Always)
            .size(
                [settings.width as f32, settings.height as f32],
                imgui::Condition::Always,
            )
            .flags(
                WindowFlags::NO_TITLE_BAR
                    | WindowFlags::NO_RESIZE
                    | WindowFlags::NO_SCROLLBAR
                    // | WindowFlags::NO_INPUTS
                    | WindowFlags::NO_SAVED_SETTINGS
                    | WindowFlags::NO_FOCUS_ON_APPEARING
                    | WindowFlags::NO_BRING_TO_FRONT_ON_FOCUS,
            )
            .build(|| {
                let new_transform = ui_state
                    .gizmo
                    .gizmo(ui)
                    .size(75.0)
                    .operation(ui_state.gizmo_operation)
                    .mode(ui_state.gizmo_mode)
                    .manipulate(
                        scene.objects[1].transform.clone(),
                        scene
                            .camera
                            .get_projection_matrix(settings.width as f32, settings.height as f32),
                        scene.camera.get_view_matrix(),
                    );

                if let Some(new_transform) = new_transform {
                    scene.objects[1].transform = new_transform;
                    //println!("{:#?}", new_transform);
                }
            });
    });

    ui.window("Camera").build(|| {
        transform_controls(ui, &mut scene.camera.transform, 100, ui_state);
        imgui::Drag::new("near").build(ui, &mut scene.camera.inner.near);
        imgui::Drag::new("far").build(ui, &mut scene.camera.inner.far);

        match &mut scene.camera.inner.projection {
            hikari_3d::Projection::Perspective(fov) => {
                imgui::Drag::new("fov").build(ui, fov);
            }
            hikari_3d::Projection::Orthographic => todo!(),
        }

        imgui::Drag::new("exposure").build(ui, &mut scene.camera.inner.exposure);

        ui.checkbox("vsync", &mut settings.vsync);
        ui.checkbox("fxaa", &mut settings.fxaa);
    });

    ui.window("Light").build(|| {
        transform_controls(ui, &mut scene.dir_light.transform, 101, ui_state);
    });
}

fn composite_pass(
    device: &Arc<rg::Device>,
    pbr_output: rg::GpuHandle<rg::SampledImage>,
    imgui_output: rg::GpuHandle<rg::SampledImage>,
    gb: &mut rg::GraphBuilder<Args>,
) {
    let vertex = r"
    #version 450
    vec2 positions[6] = vec2[](
        vec2(-1, -1),
        vec2(1, -1),
        vec2(1, 1),
        vec2(-1, 1),
        vec2(-1, -1),
        vec2(1, 1)
    );
    vec2 texCoords[6] = vec2[](
        vec2(0, 1),
        vec2(1, 1),
        vec2(1, 0),
        vec2(0, 0),
        vec2(0, 1),
        vec2(1, 0)
    );
    layout(location = 0) out vec2 texCoord;
    void main() {
        gl_Position = vec4(positions[gl_VertexIndex], 0.0, 1.0);
        texCoord = texCoords[gl_VertexIndex];
    }
    ";

    let fragment = r"
    #version 450
    layout(set = 0, binding = 0) uniform sampler2D pbr;
    layout(set = 0, binding = 1) uniform sampler2D imgui;
    layout(location = 0) in vec2 texCoord;
    layout(location = 0) out vec4 color;
    void main() {
        vec4 pbrColor = texture(pbr, texCoord);
        vec4 imguiColor = texture(imgui, texCoord);
        color.rgb = pbrColor.rgb * (1.0 - imguiColor.a) + imguiColor.rgb;
        color.a = 1.0;
    }
    ";
    let shader = rg::ShaderProgramBuilder::vertex_and_fragment(
        "CompositeShader",
        &rg::ShaderCode {
            entry_point: "main",
            data: rg::ShaderData::Glsl(vertex.to_string()),
        },
        &rg::ShaderCode {
            entry_point: "main",
            data: rg::ShaderData::Glsl(fragment.to_string()),
        },
    )
    .build(device)
    .expect("Failed to create composite shader");

    gb.add_renderpass(
        rg::Renderpass::<Args>::new("CompositePass", rg::ImageSize::default_xy(), move |cmd, _| {
            cmd.set_shader(&shader);
            // cmd.set_blend_state(rg::BlendState {
            //     enabled: true,
            //     src_color_blend_factor: rg::BlendFactor::SrcAlpha,
            //     dst_color_blend_factor: rg::BlendFactor::OneMinusSrcAlpha,
            //     color_blend_op: rg::BlendOp::Add,
            //     src_alpha_blend_factor: rg::BlendFactor::One,
            //     dst_alpha_blend_factor: rg::BlendFactor::Zero,
            //     alpha_blend_op: rg::BlendOp::Add
            // });
            cmd.draw(0..6, 0..1);
        })
        .sample_image(
            &pbr_output,
            rg::AccessType::FragmentShaderReadSampledImageOrUniformTexelBuffer,
            0,
        )
        .sample_image(
            &imgui_output,
            rg::AccessType::FragmentShaderReadSampledImageOrUniformTexelBuffer,
            1,
        )
        .present(),
    );
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    SimpleLogger::new()
        .without_timestamps()
        //.with_level(log::LevelFilter::Info)
        .init()
        .unwrap();
    hikari_dev::profiling_init();

    let event_loop = EventLoop::new();
    let mut window = WindowBuilder::new()
        .with_inner_size(LogicalSize::new(WIDTH, HEIGHT))
        .build(&event_loop)?;

    let imgui = imgui::Context::create();
    let mut imgui = rg::imgui_support::Backend::new(&mut window, imgui)?;
    let hidpi_factor = imgui.hidpi_factor();
    imgui
        .context()
        .fonts()
        .add_font(&[imgui::FontSource::TtfData {
            data: include_bytes!("fonts/Roboto-Regular.ttf"),
            size_pixels: (13.0 * hidpi_factor) as f32,
            config: None,
        }]);

    let mut settings = Settings {
        fxaa: true,
        vsync: true,
        width: WIDTH,
        height: HEIGHT,
    };
    let mut gfx = rg::Gfx::new(
        &window,
        rg::GfxConfig {
            debug: true,
            features: rg::Features::default(),
            vsync: settings.vsync,
            ..Default::default()
        },
    )?;

    let device = gfx.device().clone();
    let mut gb = rg::GraphBuilder::new(&mut gfx, WIDTH, HEIGHT);

    let depth_prepass_output = depth_prepass(&device, &mut gb);
    let pbr_output = pbr_pass(&device, &depth_prepass_output, &mut gb);
    let fxaa_output = fxaa_pass(&device, pbr_output, &mut gb);
    let imgui_output = imgui_pass(&device, &mut imgui, &mut gb);

    composite_pass(&device, fxaa_output, imgui_output, &mut gb);

    let mut graph = gb.build()?;

    //let mut sponza = load_mesh(&device, "../../assets/models/sponza/sponza.glb")?;
    let mut sponza = load_mesh(&device, "../../assets/models/sponza/sponza.glb")?;
    let mut helmet = load_mesh(&device, "../../assets/models/cerberus/cerberus.gltf")?;
    //let mut sponza =  load_mesh(&device, "../../assets/models/cube.glb")?;
    let mut scene = Scene {
        objects: vec![
            GameObject::new(&Arc::new(sponza.pop().expect("No model found"))),
            GameObject::new(&Arc::new(helmet.pop().expect("No model found"))),
        ],
        camera: Camera {
            transform: Transform {
                position: hikari_math::Vec3::ZERO,
                rotation: hikari_math::Quat::IDENTITY,
                // rotation: hikari_math::Quat::from_axis_angle(
                //     hikari_math::Vec3::Y,
                //     std::f32::consts::FRAC_PI_2,
                // ),
                scale: hikari_math::Vec3::ONE,
            },
            inner: hikari_3d::Camera {
                near: 0.1,
                far: 10000.0,
                exposure: 1.0,
                projection: hikari_3d::Projection::Perspective(45.0),
                is_primary: true
            },
        },
        dir_light: Light {
            light: hikari_3d::Light {
                color: hikari_math::Vec4::ONE,
                intensity: 1.0,
                size: 1.0,
                shadow: None,
                kind: hikari_3d::LightKind::Directional,
            },
            transform: Transform::default(),
        },
    };
    // proj_view.project_point3()
    // println!("{:?}", proj_view);
    // panic!();
    let mut ui_state = UiState {
        gizmo: imgui::gizmo::GizmoContext::new(),
        gizmo_operation: imgui::gizmo::Operation::Translate,
        gizmo_mode: imgui::gizmo::Mode::World,

        euler_cache: HashMap::new(),
    };

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        imgui.handle_event(&window, &event);
        match event {
            Event::RedrawRequested(_) => {
                hikari_dev::profile_scope!("mainloop");
                let draw_data = imgui.new_frame(&window, |ui| {
                    imgui_update(ui, &mut scene, &mut ui_state, &mut settings);
                });

                // I hate this too, but winit won't allow me to get a non static reference to DrawData
                // I don't understand what the problem here is
                let draw_data = unsafe { std::mem::transmute(draw_data) };

                gfx.set_vsync(settings.vsync);

                graph.execute((&scene, &settings, draw_data)).unwrap();

                hikari_dev::finish_frame!();
            }
            Event::MainEventsCleared => {
                window.request_redraw();
            }
            Event::WindowEvent {
                event,
                window_id: _,
            } => match event {
                WindowEvent::Resized(size) => {
                    gfx.resize(size.width, size.height)
                        .expect("Failed to resize graphics context");
                    graph
                        .resize(size.width, size.height)
                        .expect("Failed to resize graph");
                    settings.width = size.width;
                    settings.height = size.height;
                }
                WindowEvent::CloseRequested => {
                    println!("Closing");
                    *control_flow = ControlFlow::Exit;
                }
                _ => {}
            },
            Event::LoopDestroyed => {
                graph.prepare_exit();
                let _ = device.clone(); //winit doesn't drop anything that's not passed to this closure so this is necessary to drop the device
            }
            _ => (),
        }
    })
}

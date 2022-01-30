use itertools::izip;
use simple_logger::SimpleLogger;
use std::sync::Arc;
use winit::{
    dpi::{LogicalSize},
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

use hikari_render as rg;
pub use hikari_3d::texture::*;
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
    albedo_factor: glam::Vec4,
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
struct Transform {
    pub position: glam::Vec3,
    pub rotation: glam::Quat,
    pub scale: glam::Vec3,
}
impl Default for Transform {
    fn default() -> Self {
        Self {
            position: glam::Vec3::ZERO,
            rotation: glam::Quat::IDENTITY,
            scale: glam::Vec3::ONE,
        }
    }
}
struct GameObject {
    model: Arc<Model>,
    transform: Transform,
}

impl GameObject {
    pub fn new(model: &Arc<Model>) -> Self {
        Self {
            model: model.clone(),
            transform: Transform::default(),
        }
    }
}
enum Projection {
    Perspective(f32),
    Orthographic,
}

impl Projection {
    pub fn get_matrix(&self, near: f32, far: f32, width: u32, height: u32) -> glam::Mat4 {
        match self {
            Projection::Perspective(fov) => glam::Mat4::perspective_lh(
                fov.to_radians(),
                width as f32 / height as f32,
                near,
                far,
            ),
            Projection::Orthographic => {
                todo!()
            }
        }
    }
}

struct Args<'imgui> {
    draw_data: &'imgui imgui::DrawData,
    fxaa: bool,
    width: u32,
    height: u32,
}
struct Camera {
    transform: Transform,
    near: f32,
    far: f32,
    exposure: f32,
    projection: Projection,
}
struct Scene {
    objects: Vec<GameObject>,
    camera: Camera,
}

fn load_mesh(
    device: &Arc<rg::Device>,
    path: &str,
) -> Result<Vec<Model>, Box<dyn std::error::Error>> {
    let scene = hikari_asset::Scene::load(path)?;
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
            aniso_level: 16,
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
    gb: &mut rg::GraphBuilder<Scene, Args, ()>,
) -> rg::Handle<rg::SampledImage> {
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
            rg::ImageSize::default(),
        )
        .expect("Failed to create depth image");

    #[repr(C)]
    #[derive(Copy, Clone)]
    struct UBO {
        view_proj: glam::Mat4,
    }

    let mut ubo = rg::PerFrame::new([
        rg::create_uniform_buffer::<UBO>(device, 1).unwrap(),
        rg::create_uniform_buffer::<UBO>(device, 1).unwrap(),
    ]);

    #[repr(C)]
    #[derive(Copy, Clone)]
    struct PushConstants {
        transform: glam::Mat4,
    }
    let layout = rg::VertexInputLayout::new()
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
        rg::Renderpass::new(
            "DepthPrepass",
            rg::ImageSize::default(),
            move |cmd, scene: &Scene, _: &_, _| {
                cmd.set_shader(&shader);
                cmd.set_vertex_input_layout(layout);

                cmd.set_depth_stencil_state(rg::DepthStencilState {
                    depth_test_enabled: true,
                    depth_write_enabled: true,
                    depth_compare_op: rg::CompareOp::Less,
                    ..Default::default()
                });

                let proj = scene.camera.projection.get_matrix(
                    scene.camera.near,
                    scene.camera.far,
                    WIDTH,
                    HEIGHT,
                );
                let cam_transform = glam::Mat4::from_rotation_translation(
                    scene.camera.transform.rotation,
                    scene.camera.transform.position,
                );
                let view_proj = proj * cam_transform.inverse();

                ubo.get_mut().mapped_slice_mut()[0] = UBO { view_proj };

                cmd.set_uniform_buffer(ubo.get(), 0..1, 0, 0);

                for object in &scene.objects {
                    let transform = glam::Mat4::from_scale_rotation_translation(
                        object.transform.scale,
                        object.transform.rotation,
                        object.transform.position,
                    );
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

                        cmd.draw_indexed(0..mesh.indices.len(), 0, 0..1);
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
    depth_prepass: &rg::Handle<rg::SampledImage>,
    gb: &mut rg::GraphBuilder<Scene, Args, ()>,
) -> rg::Handle<rg::SampledImage> {
    #[repr(C)]
    #[derive(Debug, Copy, Clone)]
    struct Material {
        albedo: glam::Vec4,
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
        transform: glam::Mat4,
        material: Material,
    }

    #[repr(C)]
    #[derive(Copy, Clone)]
    struct UBO {
        camera_position: glam::Vec3A,
        view_proj: glam::Mat4,
        exposure: f32,
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
        hikari_asset::image::load_from_file("examples/checkerboard.png")
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
            aniso_level: 9,
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
            aniso_level: 0,
            generate_mips: false,
        },
    )
    .expect("Failed to create black texture");

    let black = Arc::new(black);

    let mut ubo = rg::PerFrame::new([
        rg::create_uniform_buffer::<UBO>(device, 1).unwrap(),
        rg::create_uniform_buffer::<UBO>(device, 1).unwrap(),
    ]);

    let layout = rg::VertexInputLayout::new()
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
            rg::ImageSize::default(),
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
        rg::Renderpass::new(
            "PBR",
            rg::ImageSize::default(),
            move |cmd, scene: &Scene, _, _| {
                let proj = scene.camera.projection.get_matrix(
                    scene.camera.near,
                    scene.camera.far,
                    WIDTH,
                    HEIGHT,
                );
                let cam_transform = glam::Mat4::from_rotation_translation(
                    scene.camera.transform.rotation,
                    scene.camera.transform.position,
                );
                let view_proj = proj * cam_transform.inverse();

                ubo.get_mut().mapped_slice_mut()[0] = UBO {
                    view_proj,
                    camera_position: glam::Vec3A::ZERO,
                    exposure: scene.camera.exposure,
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
                        let transform = glam::Mat4::from_scale_rotation_translation(
                            object.transform.scale,
                            object.transform.rotation,
                            object.transform.position,
                        );
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

                            cmd.draw_indexed(0..mesh.indices.len(), 0, 0..1);
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
    pbr_pass: rg::Handle<rg::SampledImage>,
    gb: &mut rg::GraphBuilder<Scene, Args, ()>,
) -> rg::Handle<rg::SampledImage> {
    let vertex = r"
    #version 450

    vec2 positions[6] = vec2[](
        vec2(1, 1),
        vec2(1, -1),
        vec2(-1, -1),
        vec2(1, 1),
        vec2(-1, -1),
        vec2(-1, 1)
    );

    vec2 texCoords[6] = vec2[](
        vec2(1, 0),
        vec2(1, 1),
        vec2(0, 1),
        vec2(1, 0),
        vec2(0, 1),
        vec2(0, 0)
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
        res: glam::Vec2,
        enabled: i32,
    }
    let output = gb
        .create_image(
            "fxaa_output",
            rg::ImageConfig::color2d(),
            rg::ImageSize::default(),
        )
        .expect("Failed to create fxaa output");
    gb.add_renderpass(
        rg::Renderpass::new(
            "FXAA",
            rg::ImageSize::default(),
            move |cmd, _, args: &Args, _| {
                cmd.set_shader(&shader);

                cmd.push_constants(
                    &PushConstants {
                        res: glam::vec2(args.width as f32, args.height as f32),
                        enabled: args.fxaa as _,
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
    gb: &mut rg::GraphBuilder<Scene, Args, ()>,
) -> rg::Handle<rg::SampledImage> {
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
            rg::ImageSize::default(),
        )
        .expect("Failed to create imgui image");
    gb.add_renderpass(
        rg::Renderpass::new(
            "ImguiRenderer",
            rg::ImageSize::default(),
            move |cmd, _: &Scene, args: &Args, _| {
                renderer
                    .render(cmd.raw(), args.draw_data)
                    .expect("Failed to render imgui");
            },
        )
        .draw_image(&imgui_image, rg::AttachmentConfig::color_default(0)),
    );

    imgui_image
}
fn transform_controls(ui: &imgui::Ui, transform: &mut Transform) {
    let mut position: [f32; 3] = transform.position.into();
    imgui::Drag::new("position").build_array(ui, &mut position);
    transform.position = position.into();

    let (x, y, z) = transform.rotation.to_euler(glam::EulerRot::XYZ);
    let mut euler_xyz = [x, y, z].map(|x| x.to_degrees());
    imgui::Drag::new("rotation")
        .range(0.0001, 89.999)
        .build_array(ui, &mut euler_xyz);
    let quat = glam::Quat::from_euler(
        glam::EulerRot::XYZ,
        euler_xyz[0].to_radians(),
        euler_xyz[1].to_radians(),
        euler_xyz[2].to_radians(),
    );
    transform.rotation = quat;

    let mut scale: [f32; 3] = transform.scale.into();
    imgui::Drag::new("scale").build_array(ui, &mut scale);
    transform.scale = scale.into();
}
fn imgui_update(ui: &imgui::Ui, scene: &mut Scene, vsync: &mut bool, fxaa: &mut bool) {
    //ui.show_demo_window(&mut true);

    ui.window("Camera").build(|| {
        transform_controls(ui, &mut scene.camera.transform);
        imgui::Drag::new("near").build(ui, &mut scene.camera.near);
        imgui::Drag::new("far").build(ui, &mut scene.camera.far);

        match &mut scene.camera.projection {
            Projection::Perspective(fov) => {
                imgui::Drag::new("fov").build(ui, fov);
            },
            Projection::Orthographic => todo!(),
        }

        imgui::Drag::new("exposure").build(ui, &mut scene.camera.exposure);

        ui.checkbox("vsync", vsync);
        ui.checkbox("fxaa", fxaa);
    });

    ui.window("Light").build(|| {
        transform_controls(ui, &mut scene.camera.transform);
    });
}

fn composite_pass(
    device: &Arc<rg::Device>,
    pbr_output: rg::Handle<rg::SampledImage>,
    imgui_output: rg::Handle<rg::SampledImage>,
    gb: &mut rg::GraphBuilder<Scene, Args, ()>,
) {
    let vertex = r"
    #version 450

    vec2 positions[6] = vec2[](
        vec2(1, 1),
        vec2(1, -1),
        vec2(-1, -1),
        vec2(1, 1),
        vec2(-1, -1),
        vec2(-1, 1)
    );

    vec2 texCoords[6] = vec2[](
        vec2(1, 0),
        vec2(1, 1),
        vec2(0, 1),
        vec2(1, 0),
        vec2(0, 1),
        vec2(0, 0)
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
        rg::Renderpass::new(
            "CompositePass",
            rg::ImageSize::default(),
            move |cmd, _: &Scene, _: &Args, _| {
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
            },
        )
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

    let event_loop = EventLoop::new();
    let mut window = WindowBuilder::new()
        .with_inner_size(LogicalSize::new(WIDTH, HEIGHT))
        .build(&event_loop)?;

    let imgui = rg::imgui::Context::create();
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

    let mut vsync = true;
    let mut gfx = rg::Gfx::new(
        &window,
        rg::GfxConfig {
            debug: true,
            features: rg::Features::default(),
            vsync,
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

    let mut sponza = load_mesh(&device, "../../assets/models/sponza/sponza.glb")?;

    let mut scene = Scene {
        objects: vec![GameObject::new(&Arc::new(
            sponza.pop().expect("No model found"),
        ))],

        camera: Camera {
            transform: Transform {
                position: glam::Vec3::ZERO,
                rotation: glam::Quat::from_axis_angle(glam::Vec3::Y, std::f32::consts::FRAC_PI_2),
                scale: glam::Vec3::ONE,
            },
            near: 0.1,
            far: 10_000.0,
            exposure: 1.0,
            projection: Projection::Perspective(45.0),
        },
    };
    let mut dt = 0.0;
    let mut fxaa = true;
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        imgui.handle_event(&window, &event);
        match event {
            Event::RedrawRequested(_) => {
                hikari_dev::profile_scope!("mainloop");
                let now = std::time::Instant::now();

                let draw_data = imgui.new_frame(&window, |ui| {
                    imgui_update(ui, &mut scene, &mut vsync, &mut fxaa);
                });

                // I hate this too, but winit won't allow me to get a non static reference to DrawData
                // I don't understand what the problem here is
                let draw_data = unsafe { std::mem::transmute(draw_data) };

                gfx.set_vsync(vsync);

                graph
                    .execute(
                        &mut gfx,
                        &scene,
                        &Args {
                            draw_data,
                            fxaa,
                            width: window.inner_size().width,
                            height: window.inner_size().height,
                        },
                        &(),
                    )
                    .unwrap();

                //scene.objects[0].position.y += 1.0 * dt;
                dt = now.elapsed().as_secs_f32();
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

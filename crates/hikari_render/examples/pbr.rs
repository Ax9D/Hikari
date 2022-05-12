use hikari_3d::{Projection, Scene};
use hikari_asset::{AssetManager, AssetManagerBuilder, Assets};
use simple_logger::SimpleLogger;
use std::sync::Arc;
use winit::{
    dpi::LogicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

pub use hikari_3d::texture::*;
use hikari_math::*;
use hikari_render as rg;

mod common;

const WIDTH: u32 = 1280;
const HEIGHT: u32 = 720;
struct Params {
    fxaa: bool,
    width: u32,
    height: u32,
}

// fn load_mesh(
//     device: &Arc<rg::Device>,
//     path: &str,
// ) -> Result<Vec<Model>, Box<dyn std::error::Error>> {
//     let scene = Scene::load(path)?;
//     let mut textures = Vec::new();
//     for texture in &scene.textures {
//         let data = &texture.data;

//         // let mut path = String::from("/home/atri/test_img/");
//         // path.push_str(texture.name());
//         // path.push_str(".png");
//         // save_image(&path, data, texture.width(), texture.height());

//         let config = TextureConfig {
//             format: texture.format,
//             filtering: FilterMode::Linear,
//             wrap_x: texture.wrap_x,
//             wrap_y: texture.wrap_y,
//             aniso_level: 16,
//             generate_mips: texture.generate_mips,
//         };

//         let texture = Texture2D::new(device, data, texture.width, texture.height, config)?;

//         textures.push(Arc::new(texture));
//     }

//     let mut materials = Vec::new();

//     for material in &scene.materials {
//         let albedo = material
//             .albedo_map
//             .as_ref()
//             .map(|desc| textures[desc.index].clone());
//         let albedo_factor = material.albedo;

//         let metallic = material
//             .metallic_map
//             .as_ref()
//             .map(|desc| textures[desc.index].clone());

//         let metallic_factor = material.metallic;

//         let roughness = material
//             .roughness_map
//             .as_ref()
//             .map(|desc| textures[desc.index].clone());
//         let roughness_factor = material.roughness;

//         let normal = material
//             .normal_map
//             .as_ref()
//             .map(|desc| textures[desc.index].clone());

//         let albedo_set = material
//             .albedo_map
//             .as_ref()
//             .map(|desc| desc.tex_coord_set as i32)
//             .unwrap_or(-1);
//         let roughness_set = material
//             .roughness_map
//             .as_ref()
//             .map(|desc| desc.tex_coord_set as i32)
//             .unwrap_or(-1);
//         let metallic_set = material
//             .metallic_map
//             .as_ref()
//             .map(|desc| desc.tex_coord_set as i32)
//             .unwrap_or(-1);
//         let normal_set = material
//             .normal_map
//             .as_ref()
//             .map(|desc| desc.tex_coord_set as i32)
//             .unwrap_or(-1);

//         materials.push(Arc::new(Material {
//             albedo,
//             albedo_set,
//             albedo_factor,
//             roughness,
//             roughness_set,
//             roughness_factor,
//             metallic,
//             metallic_set,
//             metallic_factor,
//             normal,
//             normal_set,
//         }));
//     }
//     let mut models = Vec::new();
//     for model in &scene.models {
//         let mut meshes = Vec::new();
//         for mesh in &model.meshes {
//             let mut vertex_data = Vec::new();
//             for (&position, &normal, &tc0, &tc1) in izip!(
//                 mesh.positions.iter(),
//                 mesh.normals.iter(),
//                 mesh.texcoord0.iter(),
//                 mesh.texcoord1.iter()
//             ) {
//                 vertex_data.push(Vertex {
//                     position: position.into(),
//                     normal: normal.into(),
//                     tc0: tc0.into(),
//                     tc1: tc1.into(),
//                 });
//             }

//             let mut vertices = rg::create_vertex_buffer(device, vertex_data.len())?;
//             vertices.upload(&vertex_data, 0)?;

//             let mut indices = rg::create_index_buffer(device, mesh.indices.len())?;
//             indices.upload(&mesh.indices, 0)?;

//             meshes.push(Mesh {
//                 vertices,
//                 material: materials[mesh.material.unwrap()].clone(),
//                 indices,
//             })
//         }
//         models.push(Model { meshes });
//     }

//     Ok(models)
// }
fn depth_prepass(
    device: &Arc<rg::Device>,
    gb: &mut rg::GraphBuilder<Args>,
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
        view_proj: Mat4,
    }

    let mut ubo = rg::PerFrame::new([
        rg::create_uniform_buffer::<UBO>(device, 1).unwrap(),
        rg::create_uniform_buffer::<UBO>(device, 1).unwrap(),
    ]);

    #[repr(C)]
    #[derive(Copy, Clone)]
    struct PushConstants {
        transform: Mat4,
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
        rg::Renderpass::<Args>::new(
            "DepthPrepass",
            rg::ImageSize::default(),
            move |cmd, (cam_transform, scene, _, _, _, _)| {
                cmd.set_shader(&shader);
                cmd.set_vertex_input_layout(layout);

                cmd.set_depth_stencil_state(rg::DepthStencilState {
                    depth_test_enabled: true,
                    depth_write_enabled: true,
                    depth_compare_op: rg::CompareOp::Less,
                    ..Default::default()
                });

                let proj = scene.camera.get_projection_matrix(WIDTH, HEIGHT);

                let cam_transform =
                    Mat4::from_rotation_translation(cam_transform.rotation, cam_transform.position);
                let view_proj = proj * cam_transform.inverse();

                ubo.get_mut().mapped_slice_mut()[0] = UBO { view_proj };

                cmd.set_uniform_buffer(ubo.get(), 0..1, 0, 0);

                for mesh in &scene.meshes {
                    let transform = Mat4::IDENTITY;

                    for sub_mesh in &mesh.sub_meshes {
                        {
                            hikari_dev::profile_scope!("Set vertex and index buffers");
                            cmd.set_vertex_buffer(&sub_mesh.vertices, 0);
                            cmd.set_index_buffer(&sub_mesh.indices);
                        }

                        cmd.push_constants(&PushConstants { transform }, 0);

                        // println!(
                        //     "{:?} {:?} {:?} {:?}",
                        //     albedo.raw().image(),
                        //     roughness.raw().image(),
                        //     metallic.raw().image(),
                        //     normal.raw().image()
                        // );

                        cmd.draw_indexed(0..sub_mesh.indices.len(), 0, 0..1);
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
    gb: &mut rg::GraphBuilder<Args>,
) -> rg::Handle<rg::SampledImage> {
    #[repr(C)]
    #[derive(Debug, Copy, Clone)]
    struct Material {
        albedo: Vec4,
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
        transform: Mat4,
        material: Material,
    }

    #[repr(C)]
    #[derive(Copy, Clone)]
    struct UBO {
        camera_position: Vec3A,
        view_proj: Mat4,
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

    let (checkerboard, width, height) = hikari_3d::image::open_rgba8("examples/checkerboard.png")
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

    let default_material = hikari_3d::Material {
        albedo: None,
        albedo_set: 0,
        albedo_factor: hikari_math::Vec4::ONE,
        roughness: None,
        roughness_set: 0,
        roughness_factor: 1.0,
        metallic: None,
        metallic_set: 0,
        metallic_factor: 0.0,
        normal: None,
        normal_set: 0,
    };

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
        rg::Renderpass::<Args>::new(
            "PBR",
            rg::ImageSize::default(),
            move |cmd, (cam_transform, scene, textures, materials, _, _)| {
                let proj = scene.camera.get_projection_matrix(WIDTH, HEIGHT);
                let cam_transform =
                    Mat4::from_rotation_translation(cam_transform.rotation, cam_transform.position);
                let view_proj = proj * cam_transform.inverse();

                ubo.get_mut().mapped_slice_mut()[0] = UBO {
                    view_proj,
                    camera_position: Vec3A::ZERO,
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
                    for mesh in &scene.meshes {
                        let transform = Mat4::IDENTITY;

                        for sub_mesh in &mesh.sub_meshes {
                            {
                                hikari_dev::profile_scope!("Set vertex and index buffers");
                                cmd.set_vertex_buffer(&sub_mesh.vertices, 0);
                                cmd.set_index_buffer(&sub_mesh.indices);
                            }
                            let material = materials
                                .get(&sub_mesh.material)
                                .unwrap_or(&default_material);
                            let gpu_material = Material {
                                albedo: material.albedo_factor,
                                roughness: material.roughness_factor,
                                metallic: material.metallic_factor,
                                albedo_uv_set: material.albedo_set,
                                roughness_uv_set: material.roughness_set,
                                metallic_uv_set: material.metallic_set,
                                normal_uv_set: material.normal_set,
                            };

                            let pc = PushConstants {
                                transform,
                                material: gpu_material,
                            };

                            cmd.push_constants(&pc, 0);

                            let albedo = material
                                .albedo
                                .as_ref()
                                .map(|handle| {
                                    textures.get(handle).unwrap_or(&checkerboard.as_ref())
                                })
                                .unwrap_or(&checkerboard);
                            let roughness = material
                                .roughness
                                .as_ref()
                                .map(|handle| {
                                    textures.get(handle).unwrap_or(&checkerboard.as_ref())
                                })
                                .unwrap_or(&black);
                            let metallic = material
                                .metallic
                                .as_ref()
                                .map(|handle| {
                                    textures.get(handle).unwrap_or(&checkerboard.as_ref())
                                })
                                .unwrap_or(&black);
                            let normal = material
                                .normal
                                .as_ref()
                                .map(|handle| {
                                    textures.get(handle).unwrap_or(&checkerboard.as_ref())
                                })
                                .unwrap_or(&black);

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

                            cmd.draw_indexed(0..sub_mesh.indices.len(), 0, 0..1);
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
    gb: &mut rg::GraphBuilder<Args>,
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
        res: Vec2,
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
        rg::Renderpass::<Args>::new(
            "FXAA",
            rg::ImageSize::default(),
            move |cmd, (_, _, _, _, params, _)| {
                cmd.set_shader(&shader);

                cmd.push_constants(
                    &PushConstants {
                        res: vec2(params.width as f32, params.height as f32),
                        enabled: params.fxaa as _,
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
        rg::Renderpass::<Args>::new(
            "ImguiRenderer",
            rg::ImageSize::default(),
            move |cmd, (_, _scene, _, _, _, draw_data)| {
                renderer
                    .render(cmd.raw(), draw_data)
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

    let (x, y, z) = transform.rotation.to_euler(EulerRot::XYZ);
    let mut euler_xyz = [x, y, z].map(|x| x.to_degrees());
    imgui::Drag::new("rotation")
        .range(0.0001, 89.999)
        .build_array(ui, &mut euler_xyz);
    let quat = Quat::from_euler(
        EulerRot::XYZ,
        euler_xyz[0].to_radians(),
        euler_xyz[1].to_radians(),
        euler_xyz[2].to_radians(),
    );
    transform.rotation = quat;

    let mut scale: [f32; 3] = transform.scale.into();
    imgui::Drag::new("scale").build_array(ui, &mut scale);
    transform.scale = scale.into();
}
fn imgui_update(ui: &imgui::Ui, scene: &mut hikari_3d::Scene, vsync: &mut bool, fxaa: &mut bool) {
    //ui.show_demo_window(&mut true);

    ui.window("Camera").build(|| {
        //transform_controls(ui, &mut scene.camera.transform);
        imgui::Drag::new("near").build(ui, &mut scene.camera.near);
        imgui::Drag::new("far").build(ui, &mut scene.camera.far);

        match &mut scene.camera.projection {
            Projection::Perspective(fov) => {
                imgui::Drag::new("fov").build(ui, fov);
            }
            Projection::Orthographic => todo!(),
        }

        imgui::Drag::new("exposure").build(ui, &mut scene.camera.exposure);

        ui.checkbox("vsync", vsync);
        ui.checkbox("fxaa", fxaa);
    });

    // ui.window("Light").build(|| {
    //     transform_controls(ui, &mut scene.camera.transform);
    // });
}

fn composite_pass(
    device: &Arc<rg::Device>,
    pbr_output: rg::Handle<rg::SampledImage>,
    imgui_output: rg::Handle<rg::SampledImage>,
    gb: &mut rg::GraphBuilder<Args>,
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
        rg::Renderpass::<Args>::new("CompositePass", rg::ImageSize::default(), move |cmd, _| {
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

type Args = (
    Transform,
    Scene,
    Assets<hikari_3d::Texture2D>,
    Assets<hikari_3d::Material>,
    Params,
    imgui::DrawData,
);

fn setup_assets(
    device: &Arc<rg::Device>,
) -> (
    AssetManager,
    Assets<hikari_3d::Texture2D>,
    Assets<hikari_3d::Material>,
    Assets<hikari_3d::Scene>,
) {
    let thread_pool = Arc::new(rayon::ThreadPoolBuilder::new().build().unwrap());
    let manager = AssetManagerBuilder::new(&thread_pool);

    let mut scenes = Assets::<hikari_3d::Scene>::new();
    let mut textures = Assets::<hikari_3d::Texture2D>::new();
    let mut materials = Assets::<hikari_3d::Material>::new();
    let mut manager = AssetManagerBuilder::new(&thread_pool);
    manager.add_loader(
        hikari_3d::SceneLoader {
            device: device.clone(),
        },
        &scenes,
    );
    manager.add_loader(
        TextureLoader {
            device: device.clone(),
        },
        &textures,
    );
    manager.add_loader((), &materials);

    let ass_man = manager.build();

    (ass_man, textures, materials, scenes)
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
    let mut gfx = rg::Gfx::headless(rg::GfxConfig {
        debug: true,
        features: rg::Features::default(),
        vsync,
        ..Default::default()
    })?;

    let device = gfx.device().clone();

    let (ass_man, mut textures, mut materials, mut scenes) = setup_assets(&device);

    let mut gb = rg::GraphBuilder::new(&mut gfx, WIDTH, HEIGHT);

    let depth_prepass_output = depth_prepass(&device, &mut gb);
    let pbr_output = pbr_pass(&device, &depth_prepass_output, &mut gb);
    let fxaa_output = fxaa_pass(&device, pbr_output, &mut gb);
    let imgui_output = imgui_pass(&device, &mut imgui, &mut gb);

    composite_pass(&device, fxaa_output, imgui_output, &mut gb);

    let mut graph = gb.build()?;

    let sponza: hikari_asset::Handle<hikari_3d::Scene> =
        ass_man.load("../../assets/models/sponza/sponza.glb")?;

    let sponza_erased = sponza.clone().into();
    loop {
        ass_man.update(&mut scenes);
        if let Some(load_status) = ass_man.get_load_status(&sponza_erased) {
            if matches!(load_status, hikari_asset::LoadStatus::Loading) {
                continue;
            }
        }
        break;
    }
    println!("Loaded sponza");
    // let mut scene = Scene {
    //     objects: vec![GameObject::new(&Arc::new(
    //         sponza.pop().expect("No model found"),
    //     ))],

    //     camera: Camera {
    //         transform: Transform {
    //             position: Vec3::ZERO,
    //             rotation: Quat::from_axis_angle(Vec3::Y, std::f32::consts::FRAC_PI_2),
    //             scale: Vec3::ONE,
    //         },
    //         near: 0.1,
    //         far: 10_000.0,
    //         exposure: 1.0,
    //         projection: Projection::Perspective(45.0),
    //     },
    // };

    let mut dt = 0.0;
    let mut fxaa = true;

    let mut camera_transform = Transform {
        position: Vec3::ZERO,
        rotation: Quat::from_axis_angle(Vec3::Y, std::f32::consts::FRAC_PI_2),
        scale: Vec3::ONE,
    };
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        let mut scene = scenes.get_mut(&sponza).expect("Sponza not present");
        imgui.handle_event(&window, &event);
        match event {
            Event::RedrawRequested(_) => {
                hikari_dev::profile_scope!("mainloop");
                let now = std::time::Instant::now();

                let draw_data = imgui.new_frame(&window, |ui| {
                    imgui_update(ui, &mut scene, &mut vsync, &mut fxaa);
                });
                gfx.set_vsync(vsync);

                graph
                    .execute((
                        &camera_transform,
                        &scene,
                        &textures,
                        &materials,
                        &Params {
                            fxaa,
                            width: window.inner_size().width,
                            height: window.inner_size().height,
                        },
                        draw_data,
                    ))
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

        ass_man.update(&mut textures);
        ass_man.update(&mut materials);
        ass_man.update(&mut scenes);
    })
}

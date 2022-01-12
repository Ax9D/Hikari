use bytemuck::Zeroable;
use itertools::izip;
use simple_logger::SimpleLogger;
use std::sync::Arc;
use winit::{
    dpi::LogicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

use graphy as rg;

const WIDTH: u32 = 1280;
const HEIGHT: u32 = 720;

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 3],
    normal: [f32; 3],
    tc0: [f32; 2],
    tc1: [f32; 2],
}

struct Material {
    albedo: Option<Arc<rg::Texture2D>>,
    albedo_set: i32,
    albedo_factor: glam::Vec4,
    roughness: Option<Arc<rg::Texture2D>>,
    roughness_set: i32,
    roughness_factor: f32,
    metallic: Option<Arc<rg::Texture2D>>,
    metallic_set: i32,
    metallic_factor: f32,
    normal: Option<Arc<rg::Texture2D>>,
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
    position: glam::Vec3,
    rotation: glam::Quat,
    scale: glam::Vec3,
}

struct Camera {}

impl GameObject {
    pub fn new(model: &Arc<Model>) -> Self {
        Self {
            model: model.clone(),
            position: glam::vec3(0.0, 0.0, 0.0),
            rotation: glam::Quat::IDENTITY,
            scale: glam::Vec3::ONE,
        }
    }
}
struct Scene {
    objects: Vec<GameObject>,
}
fn save_image(path: &str, data: &[u8], width: u32, height: u32) {
    image::save_buffer(path, data, width, height, image::ColorType::Rgba8).unwrap();
}
fn load_mesh(
    device: &Arc<rg::Device>,
    path: &str,
) -> Result<Vec<Model>, Box<dyn std::error::Error>> {
    let scene = hikari_asset::Scene::load(path)?;
    let mut textures = Vec::new();
    for texture in scene.textures() {
        let data = texture.data();

        // let mut path = String::from("/home/atri/test_img/");
        // path.push_str(texture.name());
        // path.push_str(".png");
        // save_image(&path, data, texture.width(), texture.height());

        let config = rg::TextureConfig {
            format: texture.format(),
            filtering: texture.filtering(),
            wrap_x: texture.wrap_x(),
            wrap_y: texture.wrap_y(),
            aniso_level: 9,
            generate_mips: false,
        };
        
        let texture = rg::Texture2D::new(device, data, texture.width(), texture.height(), config)?;

        textures.push(Arc::new(texture));
    }

    let mut materials = Vec::new();

    for material in scene.materials() {
        let albedo = material
            .albedo_map()
            .map(|desc| textures[desc.index].clone());
        let albedo_factor = *material.albedo();

        let metallic = material
            .metallic_map()
            .map(|desc| textures[desc.index].clone());

        let metallic_factor = material.metallic();

        let roughness = material
            .roughness_map()
            .map(|desc| textures[desc.index].clone());
        let roughness_factor = material.roughness();

        let normal = material
            .normal_map()
            .map(|desc| textures[desc.index].clone());

        let albedo_set = material
            .albedo_map()
            .map(|desc| desc.tex_coord_set as i32)
            .unwrap_or(-1);
        let roughness_set = material
            .roughness_map()
            .map(|desc| desc.tex_coord_set as i32)
            .unwrap_or(-1);
        let metallic_set = material
            .metallic_map()
            .map(|desc| desc.tex_coord_set as i32)
            .unwrap_or(-1);
        let normal_set = material
            .normal_map()
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
    for model in scene.models() {
        let mut meshes = Vec::new();
        for mesh in model.meshes() {
            let mut vertex_data = Vec::new();
            for (&position, &normal, &tc0, &tc1) in izip!(
                mesh.positions().iter(),
                mesh.normals().iter(),
                mesh.texcoord0().iter(),
                mesh.texcoord1().iter()
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

            let mut indices = rg::create_index_buffer(device, mesh.indices().len())?;
            indices.upload(mesh.indices(), 0)?;

            meshes.push(Mesh {
                vertices,
                material: materials[mesh.material().unwrap()].clone(),
                indices,
            })
        }
        models.push(Model { meshes });
    }

    Ok(models)
}
fn get_proj_matrix(fov: f32, width: u32, height: u32, z_near: f32, z_far: f32) -> glam::Mat4 {
    glam::Mat4::perspective_lh(
        fov.to_radians(),
        width as f32 / height as f32,
        z_near,
        z_far,
    )
}

fn pbr_pass(device: &Arc<rg::Device>, gb: &mut rg::GraphBuilder<Scene, (), ()>) {
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
            entry_point: "main".into(),
            data: rg::ShaderData::Glsl(
                std::fs::read_to_string("examples/shaders/pbr.vert").unwrap(),
            ),
        },
        &rg::ShaderCode {
            entry_point: "main".into(),
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

    let checkerboard = rg::Texture2D::new(
        device,
        &checkerboard,
        width,
        height,
        rg::TextureConfig {
            format: rg::Format::RGBA8,
            wrap_x: rg::WrapMode::Repeat,
            wrap_y: rg::WrapMode::Repeat,
            filtering: rg::FilterMode::Linear,
            aniso_level: 9,
            generate_mips: true,
        },
    )
    .expect("Failed to create checkerboard texture");

    let checkerboard = Arc::new(checkerboard);

    let black = rg::Texture2D::new(
        device,
        &[0, 0, 0, 255],
        1,
        1,
        rg::TextureConfig {
            format: rg::Format::RGBA8,
            wrap_x: rg::WrapMode::Repeat,
            wrap_y: rg::WrapMode::Repeat,
            filtering: rg::FilterMode::Linear,
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

    gb.add_renderpass(
        rg::Renderpass::new(
            "PBR",
            rg::ImageSize::default(),
            move |cmd, scene: &Scene, _, _| {
                let proj = get_proj_matrix(90.0, WIDTH, HEIGHT, 0.1, 100.0);
                let view_proj = proj * glam::Mat4::from_translation(glam::vec3(0.0, 0.0, -2.0));
                ubo.get_mut().mapped_slice_mut()[0] = UBO {
                    view_proj,
                    camera_position: glam::Vec3A::ZERO,
                    exposure: 1.0,
                };

                cmd.set_shader(&shader);

                cmd.set_vertex_input_layout(layout);

                cmd.set_depth_stencil_state(rg::DepthStencilState {
                    depth_test_enabled: true,
                    depth_write_enabled: true,
                    depth_compare_op: rg::CompareOp::LessOrEqual,
                    ..Default::default()
                });

                cmd.set_uniform_buffer(ubo.get(), 0..1, 0, 0);

                for object in &scene.objects {
                    let transform = glam::Mat4::from_scale_rotation_translation(
                        object.scale,
                        object.rotation,
                        object.position,
                    );
                    let model = &object.model;
                    for mesh in &model.meshes {
                        cmd.set_vertex_buffer(&mesh.vertices, 0);
                        cmd.set_index_buffer(&mesh.indices);

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
                        cmd.set_image(roughness.raw(), 1, 1);
                        cmd.set_image(metallic.raw(), 1, 2);
                        cmd.set_image(normal.raw(), 1, 3);
                        cmd.set_image(albedo.raw(), 1, 0);

                        cmd.draw_indexed(0..mesh.indices.len(), 0, 0..1);
                    }
                }

                ubo.next_frame();
            },
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
    let window = WindowBuilder::new()
        .with_inner_size(LogicalSize::new(WIDTH, HEIGHT))
        .build(&event_loop)?;

    let mut gfx = rg::Gfx::new(
        &window,
        rg::GfxConfig {
            debug: true,
            features: rg::Features::default(),
        },
    )?;
    let device = gfx.device().clone();
    let mut gb = rg::GraphBuilder::new(&mut gfx, WIDTH, HEIGHT);

    pbr_pass(&device, &mut gb);

    let mut graph = gb.build()?;

    let mut sponza = load_mesh(&device, "/home/atri/sponza/sponza.glb")?;

    let scene = Scene {
        objects: vec![GameObject::new(&Arc::new(
            sponza.pop().expect("No model found"),
        ))],
    };
    let mut dt = 0.0;
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        match event {
            Event::RedrawRequested(_) => {
                hikari_dev::profile_scope!("mainloop");
                let now = std::time::Instant::now();
                graph.execute(&mut gfx, &scene, &(), &()).unwrap();
                //scene.objects[0].position.y += 1.0 * dt;
                dt = now.elapsed().as_secs_f32();
                hikari_dev::finish_frame!();
            }
            Event::MainEventsCleared => {
                window.request_redraw();
            }
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                window_id: _,
            } => {
                println!("Closing");
                *control_flow = ControlFlow::Exit;
            }
            Event::LoopDestroyed => {
                graph.prepare_exit();
            }
            _ => (),
        }
    })
}

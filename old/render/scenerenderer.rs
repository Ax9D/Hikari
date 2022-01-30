use graphy::*;

use crate::{
    core::{primitives::Transform, Scene},
    model::MeshComponent,
    render::DirectionalLight,
};
pub struct RenderSettings {
    pub fxaa: bool,
    pub exposure: f32,
}
pub struct PerFrameResources {
    viewport_width: u32,
    viewport_height: u32,
    render_settings: RenderSettings,
}
pub type SGraph = Graph<Scene, PerFrameResources, ()>;
pub type SRenderpass = Renderpass<Scene, PerFrameResources, ()>;

pub struct SceneRendererState {
    graph: SGraph,
    perframe: PerFrameResources,
}

fn lighting_pass() -> Result<SRenderpass, Box<dyn std::error::Error>> {
    let vertex_code = include_str!("../../assets/shaders/pbr.vs");

    let fragment_code = include_str!("../../assets/shaders/pbr.fs");

    use graphy::buffer::*;
    use graphy::ShaderDataType as dt;

    let pipeline = graphy::Pipeline::new(graphy::PipelineDescriptor {
        shader: graphy::ShaderProgramBuilder::vertex_and_fragment(
            "pbr",
            vertex_code,
            fragment_code,
        )
        .build()?,
        primitive_topology: Default::default(),
    });
    #[repr(C)]
    struct CameraBlock {
        position: glam::Vec3A,
        view_proj: glam::Mat4,
    }

    #[repr(C)]
    struct DirectionalLightBlock {
        intensity: f32,
        color: glam::Vec3A,
        direction: glam::Vec3A,
    }

    #[repr(C)]
    struct Lights {
        directional_light: DirectionalLightBlock,
    }
    let matrices_buffer = graphy::UniformBuffer::new(0)?;
    let lights_buffer = graphy::UniformBuffer::new(1)?;
    Ok(graphy::RenderpassBuilder::new(
        "Lighting",
        move |cmd, scene: &Scene, perframe: &PerFrameResources, _| {
            for (_, (transform, camera)) in scene
                .query::<(&Transform, &crate::render::CameraComponent)>()
                .iter()
            {
                if camera.primary {
                    let proj = camera.get_projection_matrix(
                        perframe.viewport_width as f32 / perframe.viewport_height as f32,
                    );
                    let view = camera.get_view_matrix(&transform);

                    let view_proj = proj * view;

                    let position = glam::vec3a(
                        transform.position.x,
                        transform.position.y,
                        transform.position.z,
                    );
                    cmd.update_uniform_buffer(
                        &matrices_buffer,
                        &CameraBlock {
                            view_proj,
                            position,
                        },
                    );

                    use super::DirectionalLight;
                    if let Some((_, (transform, dir_light))) = scene
                        .query::<(&Transform, &DirectionalLight)>()
                        .iter()
                        .next()
                    {
                        cmd.update_uniform_buffer(
                            &lights_buffer,
                            &Lights {
                                directional_light: DirectionalLightBlock {
                                    intensity: dir_light.intensity,
                                    color: dir_light.color.into(),
                                    direction: (transform.rotation * glam::Vec3::X).into(),
                                },
                            },
                        );
                    } else {
                        cmd.update_uniform_buffer(
                            &lights_buffer,
                            &Lights {
                                directional_light: DirectionalLightBlock {
                                    intensity: 0.0,
                                    color: glam::Vec3A::ZERO,
                                    direction: glam::Vec3A::ZERO,
                                },
                            },
                        );
                    }

                    cmd.bind_pipeline(&pipeline);
                    for (_, (transform, mesh)) in
                        scene.query::<(&Transform, &MeshComponent)>().iter()
                    {
                        // let rotation = glam::Quat::from_rotation_ypr(
                        //     transform.rotation.y,
                        //     transform.rotation.x,
                        //     transform.rotation.z,
                        // );

                        let transform = glam::Mat4::from_scale_rotation_translation(
                            transform.scale,
                            transform.rotation,
                            transform.position,
                        );

                        cmd.set_mat4f("transform", &transform);
                        for mesh in mesh.model.meshes() {
                            let material = &mesh.material;

                            let albedo;
                            let albedo_map;
                            let mut albedo_set = -1;

                            match &material.albedo {
                                crate::render::material::MaterialColor::Constant(color) => {
                                    albedo = color;
                                    albedo_map = super::texture::white();
                                }
                                crate::render::material::MaterialColor::Texture(
                                    texture,
                                    texcoord_set,
                                ) => {
                                    albedo = &glam::Vec4::ONE;
                                    albedo_map = texture;
                                    albedo_set = *texcoord_set as i32;
                                }
                            }

                            let roughness;
                            let roughness_map;
                            let mut roughness_set = -1;

                            match &material.roughness {
                                crate::render::material::MaterialValue::Constant(factor) => {
                                    roughness = *factor;
                                    roughness_map = super::texture::white();
                                }
                                crate::render::material::MaterialValue::Texture(
                                    texture,
                                    texcoord_set,
                                ) => {
                                    roughness = 1.0;
                                    roughness_map = texture;
                                    roughness_set = *texcoord_set as i32;
                                }
                            }

                            let metallic;
                            let metallic_map;
                            let mut metallic_set = -1;

                            match &material.metallic {
                                crate::render::material::MaterialValue::Constant(factor) => {
                                    metallic = *factor;
                                    metallic_map = super::texture::white();
                                }
                                crate::render::material::MaterialValue::Texture(
                                    texture,
                                    texcoord_set,
                                ) => {
                                    metallic = 1.0;
                                    metallic_map = texture;
                                    metallic_set = *texcoord_set as i32;
                                }
                            }

                            let normal_map;
                            let mut normal_set = -1;

                            match &material.normal {
                                crate::render::material::MaterialValue::Constant(factor) => {
                                    normal_map = super::texture::white();
                                }
                                crate::render::material::MaterialValue::Texture(
                                    texture,
                                    texcoord_set,
                                ) => {
                                    normal_map = texture;
                                    normal_set = *texcoord_set as i32;
                                }
                            }
                            use std::ops::Deref;

                            cmd.set_vec4f("albedo", albedo.x, albedo.y, albedo.z, albedo.w);
                            cmd.set_texture("albedoMap", albedo_map.deref());
                            cmd.set_int("albedoUVSet", albedo_set);

                            //println!("Set Albedo successful");
                            cmd.set_float("roughness", roughness);
                            cmd.set_texture("roughnessMap", roughness_map.deref());
                            cmd.set_int("roughnessUVSet", roughness_set);
                            //println!("Set Roughness successful");
                            cmd.set_float("metallic", metallic);

                            cmd.set_texture("metallicMap", metallic_map.deref());
                            cmd.set_int("metallicUVSet", metallic_set);

                            cmd.set_texture("normalMap", normal_map.deref());
                            cmd.set_int("normalUVSet", normal_set);

                            //cmd.setFloat("exposure", perframe.renderSettings.exposure);

                            //println!("Set Metallic successful");
                            cmd.bind_vertex_array(&mesh.vertex_array);
                            cmd.draw_indexed(mesh.n_indices, 0);
                        }
                    }
                    return;
                }
            }

            log::warn!("No camera");
        },
    )
    .color_output(
        "lightOutput",
        graphy::ColorFormat::RGBA32F,
        false,
        graphy::ImageSize::Relative,
    )
    .color_output(
        "debugNormals",
        graphy::ColorFormat::RGBA32F,
        false,
        graphy::ImageSize::Relative,
    )
    .depth_stencil_output(
        "defaultDepth",
        graphy::DepthStencilFormat::Depth24Stencil8,
        true,
        graphy::ImageSize::Relative,
    )
    .build()?)
}
fn postprocess() -> Result<SRenderpass, Box<dyn std::error::Error>> {
    use graphy::ShaderDataType as dt;

    let quad = VertexArray::create()
        .vertex_buffer(
            &ImmutableVertexBuffer::with_data(
                &[
                    1.0_f32, 1.0, 0.0, 1.0, -1.0, 0.0, -1.0, -1.0, 0.0, -1.0, 1.0, 0.0,
                ],
                &[dt::Vec3f],
            )
            .unwrap(),
        )
        .vertex_buffer(
            &ImmutableVertexBuffer::with_data(
                &[1.0_f32, 1.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0],
                &[dt::Vec2f],
            )
            .unwrap(),
        )
        .index_buffer(&IndexBuffer::with_data(&[0, 2, 1, 0, 3, 2]).unwrap())
        .build()
        .unwrap();

    let vertex_code = include_str!("../../assets/shaders/quad.vs");

    let fragment_code = include_str!("../../assets/shaders/fxaa.fs");

    let shader =
        graphy::ShaderProgramBuilder::vertex_and_fragment("fxaa", vertex_code, fragment_code)
            .build()?;

    let pipeline = graphy::Pipeline::new(graphy::PipelineDescriptor {
        shader,
        primitive_topology: graphy::PrimitiveTopology::default(),
    });

    Ok(
        graphy::RenderpassBuilder::new("FXAA", move |cmd, _, pf: &PerFrameResources, _| {
            cmd.bind_vertex_array(&quad);
            cmd.bind_pipeline(&pipeline);

            cmd.set_uint("enabled", pf.render_settings.fxaa.into());
            cmd.set_vec2f(
                "resolution",
                pf.viewport_width as f32,
                pf.viewport_height as f32,
            );

            cmd.draw_indexed(6, 0);
        })
        .color_input("lightOutput")
        .color_output(
            "offscreen",
            ColorFormat::RGB32F,
            true,
            graphy::ImageSize::Relative,
        )
        .build()?,
    )
}
pub fn init<'a>(
    gfx: &graphy::Gfx,
    window: &mut crate::window::Window,
) -> Result<SceneRendererState, Box<dyn std::error::Error>> {
    let vertex_code = r"
    #version 450 core

    layout (location = 0) in vec3 position;
    layout (location = 1) in vec2 tcIn;

    out vec2 tc;

    void main() {
        gl_Position = vec4(position, 1.0);
        tc = tcIn;
    }
    ";
    let fragment_code = r"
    #version 450 core

    layout(binding = 0) uniform sampler2D offscreen;

    layout(location = 0) out vec4 SCREEN_COLOR;


    in vec2 tc;
    void main() {
        SCREEN_COLOR = vec4(texture(offscreen, tc).rrr,1.0);
    }
    ";

    let (width, height) = window.get_size();
    use graphy::buffer::*;
    use graphy::ShaderDataType as dt;
    let quad = VertexArray::create()
        .vertex_buffer(
            &ImmutableVertexBuffer::with_data(
                &[
                    1.0_f32, 1.0, 0.0, 1.0, -1.0, 0.0, -1.0, -1.0, 0.0, -1.0, 1.0, 0.0,
                ],
                &[dt::Vec3f],
            )
            .unwrap(),
        )
        .vertex_buffer(
            &ImmutableVertexBuffer::with_data(
                &[1.0_f32, 1.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0],
                &[dt::Vec2f],
            )
            .unwrap(),
        )
        .index_buffer(&IndexBuffer::with_data(&[0, 1, 3, 1, 2, 3]).unwrap())
        .build()
        .unwrap();

    let pipeline = graphy::Pipeline::new(graphy::PipelineDescriptor {
        shader: graphy::ShaderProgramBuilder::vertex_and_fragment(
            "generic",
            vertex_code,
            fragment_code,
        )
        .build()?,
        primitive_topology: Default::default(),
    });

    let final_ = graphy::RenderpassBuilder::new("final", move |cmd, scene, _, _| {
        //cmd.drawIndexed(6, 0);
        //cmd.bindVertexArray(&quad);
        //cmd.bindPipeline(&pipeline);
    })
    .color_input("offscreen")
    .color_output(
        "SCREEN_COLOR",
        graphy::ColorFormat::RGB8,
        true,
        graphy::ImageSize::Relative,
    )
    .build()?;

    let graph = graphy::GraphBuilder::new(width, height)
        .add_renderpass(lighting_pass()?)
        .add_renderpass(postprocess()?)
        .add_renderpass(final_)
        .build()?;

    Ok(SceneRendererState {
        graph,
        perframe: PerFrameResources {
            viewport_width: width,
            viewport_height: height,
            render_settings: RenderSettings {
                fxaa: true,
                exposure: 0.5,
            },
        },
    })
}
pub fn on_viewport_resize(ctx: &mut crate::Context, width: u32, height: u32) {
    ctx.renderer.perframe.viewport_width = width;
    ctx.renderer.perframe.viewport_height = height;

    ctx.renderer.graph.resize(width, height).unwrap();
}
pub fn draw_scene(ctx: &mut crate::Context, scene: &mut crate::core::Scene) {
    let render_data = &ctx.renderer.perframe;
    ctx.renderer
        .graph
        .execute(&mut ctx.gfx, scene, &render_data, &());
}
pub fn graph(ctx: &mut crate::Context) -> &SGraph {
    &ctx.renderer.graph
}

pub fn render_settings_mut(ctx: &mut crate::Context) -> &mut RenderSettings {
    &mut ctx.renderer.perframe.render_settings
}

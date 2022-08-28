use std::{sync::Arc, f32::consts::TAU};

use crate::{Args, world::WorldUBO};
use hikari_3d::*;
use hikari_math::*;
use hikari_render::*;
use rand::Rng;

pub const MAX_SHADOW_CASCADES: usize = 4;
pub const SHADOW_MAP_SIZE: u32 = 2048;

struct Defaults {
    noise: SampledImage
}

impl Defaults {
    pub fn new(device: &Arc<Device>) -> anyhow::Result<Self> {
        let config = ImageConfig {
            format: vk::Format::R32G32_SFLOAT,
            filtering: vk::Filter::NEAREST,
            wrap_x: vk::SamplerAddressMode::REPEAT,
            wrap_y: vk::SamplerAddressMode::REPEAT,
            wrap_z: vk::SamplerAddressMode::REPEAT,
            aniso_level: 0.0,
            mip_levels: 1,
            mip_filtering: vk::SamplerMipmapMode::NEAREST,
            usage: vk::ImageUsageFlags::SAMPLED,
            image_type: vk::ImageType::TYPE_3D,
            image_view_type: vk::ImageViewType::TYPE_3D,
            host_readable: false,
        };

        const N: usize = 32;

        let mut data: [Vec2; N * N * N] = [Vec2::ZERO; N * N * N];

        let mut rng = rand::thread_rng();

        for i in 0..N {
            for j in 0..N {
                for k in 0..N {
                    let theta = rng.gen_range(0.0..TAU);
                    data[(i * N + j) * N + k] = Vec2::new(f32::cos(theta), f32::sin(theta));

                }
            }
        }

        let data = unsafe { std::slice::from_raw_parts(data.as_ptr() as *const u8, data.len() * 2 * 4) };

        Ok(Self {
            noise: SampledImage::with_rgba8(device, data, N as u32, N as u32, N as u32, config)?
        })

    }
}
#[repr(C)]
#[derive(Copy, Clone)]
struct PushConstants {
    transform: hikari_math::Mat4,
    cascade_ix: u32
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct ShadowSplitsUBO {
}
pub fn compute_cascades(n_splits: usize, shadow_info: &ShadowInfo, light_transform: &Transform, camera: &Camera, camera_view_proj: &Mat4, world_ubo: &mut WorldUBO) {
    assert!(n_splits <= MAX_SHADOW_CASCADES);
    
    let mut splits = [0_f32; MAX_SHADOW_CASCADES];

    let near_clip = camera.near;
    let far_clip = camera.far;
    let clip_range = far_clip - near_clip;

    let min_z = near_clip;
    let max_z = near_clip + clip_range;

    let range = max_z - min_z;
    let ratio = max_z / min_z;

    for i in 0..n_splits {
        let p = (i + 1) as f32 / n_splits as f32;
        let log = min_z * ratio.powf(p);
        let uniform = min_z + range * p;
        let d = shadow_info.cascade_split_lambda * (log - uniform) + uniform;
        splits[i] = (d - near_clip) / clip_range;
    }
    
    let mut frustum_corners = [
        vec3(-1.0,  1.0, 0.0),
        vec3( 1.0,  1.0, 0.0),
        vec3(-1.0, -1.0, 0.0),
        vec3(-1.0, -1.0, 0.0),
        vec3(-1.0,  1.0, 1.0),
        vec3( 1.0,  1.0, 1.0),
        vec3( 1.0, -1.0, 1.0),
        vec3(-1.0, -1.0, 1.0)
    ];

    let view_proj_inv = camera_view_proj.inverse();

    for corner in &mut frustum_corners {
        let mut corner_world = view_proj_inv * Vec4::from((*corner, 1.0));
        corner_world /= corner_world.w;
        *corner = corner_world.xyz();
    }

    let mut last_split_dist = 0.0;

    for i in 0..n_splits {
        let mut current_frustum_corners = frustum_corners.clone();
        let current_split_dist = splits[i];

        for i in 0..4 {
            let dist = current_frustum_corners[i + 4] - current_frustum_corners[i];
            current_frustum_corners[i + 4] = current_frustum_corners[i] + (dist * current_split_dist);
            current_frustum_corners[i] = current_frustum_corners[i] + (dist * last_split_dist);
        }

        let frustum_center = current_frustum_corners.iter().sum::<Vec3>() / 8.0;

        let mut radius = 0.0;

        for corner in current_frustum_corners {
            let dist = corner.distance(frustum_center);
            radius = f32::max(radius, dist);
        }

        radius = (radius * 16.0).ceil() / 16.0;

        let light_dir = light_transform.forward();
        
        //Fixes shimmering by moving the frustum center by texel size increments
        let texels_per_unit = SHADOW_MAP_SIZE as f32 / (radius * 2.0);
        let lookat = Mat4::from_scale(Vec3::splat(texels_per_unit)) * Mat4::look_at_rh(Vec3::ZERO, -light_dir, light_transform.up());
        let lookat_inv = lookat.inverse();

        let mut frustum_center = lookat.transform_vector3(frustum_center);
        frustum_center.x = f32::floor(frustum_center.x);
        frustum_center.y = f32::floor(frustum_center.y);
        frustum_center = lookat_inv.transform_vector3(frustum_center);

        let max_extents = Vec3::splat(radius);
        let min_extents = -max_extents;

        let light_view = Mat4::look_at_rh(frustum_center - light_dir * max_extents.z, frustum_center, light_transform.up());

        let near = 0.0;
        let far = max_extents.z - min_extents.z;
        let light_ortho = Mat4::orthographic_rh(min_extents.x, max_extents.x, min_extents.y, max_extents.y, near, far);

        world_ubo.dir_light.cascades[i].split_depth = (near_clip + current_split_dist * clip_range) * -1.0;
        world_ubo.dir_light.cascades[i].view = light_view.to_cols_array();
        world_ubo.dir_light.cascades[i].view_proj = (light_ortho * light_view).to_cols_array();
        world_ubo.dir_light.cascades[i].near = near;
        world_ubo.dir_light.cascades[i].far = far;

        last_split_dist = current_split_dist;
    }

}
fn build_single_pass(
    device: &Arc<Device>,
    graph: &mut GraphBuilder<Args>,
    shader_lib: &mut ShaderLibrary,
    cascade_ix: usize,
) -> anyhow::Result<GpuHandle<SampledImage>> {

    let mut config = ImageConfig::depth_only(device);
    config.wrap_x = vk::SamplerAddressMode::CLAMP_TO_BORDER;
    config.wrap_y = config.wrap_x;

    let shadow_map = graph.create_image(
        &format!("DirectionalShadowMapCascade{}", cascade_ix),
        config,
        ImageSize::absolute_xy(SHADOW_MAP_SIZE, SHADOW_MAP_SIZE),
    )?;

    let layout = VertexInputLayout::builder()
        .buffer(
            &[
                ShaderDataType::Vec3f,
                ShaderDataType::Vec3f,
                ShaderDataType::Vec2f,
                ShaderDataType::Vec2f,
            ],
            StepMode::Vertex,
        )
    .build();

    shader_lib.insert("shadow")?;
    graph.add_renderpass(
        Renderpass::<Args>::new(
            &format!("ShadowCascade{}", cascade_ix),
            ImageSize::absolute_xy(SHADOW_MAP_SIZE, SHADOW_MAP_SIZE),
            move |cmd, (world, res, shader_lib, assets)| {
                cmd.set_shader(shader_lib.get("shadow").unwrap());

                let dir_light = res.directional_light;

                if let Some(dir_light) = dir_light {

                    let light = world.get_component::<&Light>(dir_light).unwrap();
                    if light.shadow.is_none() {
                        return;
                    }

                    cmd.set_vertex_input_layout(layout);

                    cmd.set_depth_stencil_state(DepthStencilState {
                        depth_test_enabled: true,
                        depth_write_enabled: true,
                        depth_compare_op: CompareOp::LessOrEqual,
                        ..Default::default()
                    });

                    cmd.set_rasterizer_state(RasterizerState {
                        cull_mode: CullMode::Back,
                        depth_bias_enable: true,
                        depth_bias_slope_factor: light.shadow.unwrap().constant_bias,
                        ..Default::default()
                    });
                    
                    cmd.set_uniform_buffer(res.world_ubo.get(), 0..1, 0, 0);

                    let scenes = assets.get::<Scene>().expect("Scenes pool not found");
                    for (_, (transform, mesh_comp)) in
                        &mut world.query::<(&Transform, &MeshRender)>()
                    {
                        let mut transform = transform.get_matrix();
                        match &mesh_comp.source {
                            MeshSource::Scene(handle, mesh_ix) => {
                                if let Some(scene) = scenes.get(handle) {
                                    let mesh = &scene.meshes[*mesh_ix];

                                    transform *= mesh.transform.get_matrix();

                                    cmd.push_constants(&PushConstants { transform, cascade_ix: cascade_ix as u32 }, 0);

                                    for submesh in &mesh.sub_meshes {
                                        {
                                            hikari_dev::profile_scope!(
                                                "Set vertex and index buffers"
                                            );
                                            cmd.set_vertex_buffer(&submesh.vertices, 0);
                                            cmd.set_index_buffer(&submesh.indices);
                                        }

                                        // println!(
                                        //     "{:?} {:?} {:?} {:?}",
                                        //     albedo.raw().image(),
                                        //     roughness.raw().image(),
                                        //     metallic.raw().image(),
                                        //     normal.raw().image()
                                        // );

                                        cmd.draw_indexed(0..submesh.indices.capacity(), 0, 0..1);
                                    }
                                }
                            }
                            MeshSource::None => {}
                        }
                    }
                }
            },
        )
        .draw_image(&shadow_map, AttachmentConfig::depth_only_default()),
    );
    Ok(shadow_map)
}
pub fn build_pass(
device: &Arc<Device>,
graph: &mut GraphBuilder<Args>,
shader_lib: &mut ShaderLibrary,
) -> anyhow::Result<Vec<GpuHandle<SampledImage>>>{
    let mut shadow_maps = vec![];

    for cascade_ix in 0..MAX_SHADOW_CASCADES {
        shadow_maps.push(build_single_pass(device, graph, shader_lib, cascade_ix)?);
    }

    Ok(shadow_maps)
}

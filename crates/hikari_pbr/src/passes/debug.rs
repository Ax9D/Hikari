use std::sync::Arc;

use crate::Args;
use hikari_3d::*;
use hikari_render::*;

#[allow(unused)]
pub fn build_pass(
    _device: &Arc<Device>,
    graph: &mut GraphBuilder<Args>,
    shader_lib: &mut ShaderLibrary,
    depth_map: &GpuHandle<SampledImage>,
    shadow_atlas: &GpuHandle<SampledImage>,
) -> anyhow::Result<Vec<GpuHandle<SampledImage>>> {
    let depth_debug = graph.create_image(
        "PrepassDepthDebug",
        ImageConfig::color2d_attachment(),
        ImageSize::default_xy(),
    )?;
    let shadow_atlas_debug = graph.create_image(
        "ShadowMapAtlasDebug",
        ImageConfig::color2d_attachment(),
        ImageSize::default_xy(),
    )?;

    // for (ix, _cascade) in shadow_atlas.iter().enumerate() {
    //     let image = graph.create_image(&format!("DirectionalShadowMapDebug{}", ix), ImageConfig::color2d() , ImageSize::default_xy())?;
    //     directional_shadow_debug.push(image);
    // }

    let depth_map = depth_map.clone();
    let shadow_atlas = shadow_atlas.clone();

    shader_lib.insert("debug")?;

    let pass = Renderpass::<Args>::new("Debug", ImageSize::default_xy())
        .read_image(
            &depth_map,
            AccessType::FragmentShaderReadSampledImageOrUniformTexelBuffer,
        )
        .read_image(
            &shadow_atlas,
            AccessType::FragmentShaderReadSampledImageOrUniformTexelBuffer,
        )
        .draw_image(&depth_debug, AttachmentConfig::color_default(0))
        .draw_image(&shadow_atlas_debug, AttachmentConfig::color_default(1))
        .cmd(move |cmd, graph_res, record_info, (_, _, shader_lib, ..)| {
            cmd.set_image(graph_res.get_image(&depth_map).unwrap(), 0, 0);
            cmd.set_image(graph_res.get_image(&shadow_atlas).unwrap(), 0, 1);

            cmd.set_viewport(
                0.0,
                0.0,
                record_info.framebuffer_width as f32,
                record_info.framebuffer_height as f32,
            );
            cmd.set_scissor(
                0,
                0,
                record_info.framebuffer_width,
                record_info.framebuffer_height,
            );

            cmd.set_shader(shader_lib.get("debug").unwrap());
            cmd.draw(0..6, 0..1);
        });

    // for (ix, cascade) in shadow_atlas.iter().enumerate() {
    //     pass = pass.sample_image_array(cascade, AccessType::FragmentShaderReadSampledImageOrUniformTexelBuffer, 1,  ix);
    //     pass = pass.draw_image(&directional_shadow_debug[ix], AttachmentConfig::color_default((ix + 1) as u32));
    // };

    graph.add_renderpass(pass);

    Ok(vec![depth_debug, shadow_atlas_debug])
}

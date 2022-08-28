use std::sync::Arc;

use hikari_3d::*;
use hikari_render::*;
use crate::Args;

pub fn build_pass(
    _device: &Arc<Device>,
    graph: &mut GraphBuilder<Args>,
    shader_lib: &mut ShaderLibrary,
    depth_map: &GpuHandle<SampledImage>,
    shadow_cascades: &[GpuHandle<SampledImage>],
) -> anyhow::Result<(GpuHandle<SampledImage>, Vec<GpuHandle<SampledImage>>)> {
    let depth_debug = graph.create_image("PrepassDepthDebug", ImageConfig::color2d() , ImageSize::default_xy())?;
    let mut directional_shadow_debug = vec![];

    for (ix, _cascade) in shadow_cascades.iter().enumerate() {
        let image = graph.create_image(&format!("DirectionalShadowMapDebug{}", ix), ImageConfig::color2d() , ImageSize::default_xy())?;
        directional_shadow_debug.push(image);
    }

    shader_lib.insert("debug")?;

    let mut pass = Renderpass::<Args>::new("Debug", ImageSize::default_xy(), |cmd, (_, _, shader_lib, _)| {
        cmd.set_shader(shader_lib.get("debug").unwrap());
        cmd.draw(0..6, 0..1);
        
    })
    .sample_image(depth_map, AccessType::FragmentShaderReadSampledImageOrUniformTexelBuffer, 0)
    .draw_image(&depth_debug, AttachmentConfig::color_default(0));
    
    
    for (ix, cascade) in shadow_cascades.iter().enumerate() {
        pass = pass.sample_image_array(cascade, AccessType::FragmentShaderReadSampledImageOrUniformTexelBuffer, 1,  ix);
        pass = pass.draw_image(&directional_shadow_debug[ix], AttachmentConfig::color_default((ix + 1) as u32));
    };

    graph.add_renderpass(
        pass
    );

    Ok((depth_debug, directional_shadow_debug))
}
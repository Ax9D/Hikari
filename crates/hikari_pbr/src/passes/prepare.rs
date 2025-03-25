use std::sync::Arc;

use hikari_render::{GraphBuilder, ComputePass, Buffer};
use crate::{Args, resources::RenderResources, SCENE_SET_ID};

pub fn build_pass(device: &Arc<hikari_render::Device>, graph_builder: &mut GraphBuilder<Args>) {
    let device = device.clone();
    
    graph_builder.add_computepass(ComputePass::new("Prepare").cmd(move|cmd, _, _, (_world, res, _, _)| {
        let bindless = device.bindless_resources().set().lock();
        cmd.set_bindless(*bindless);

        let res: &RenderResources = res;
        cmd.set_buffer(&res.world_ubo, 0..res.world_ubo.len(), SCENE_SET_ID, 0);
        cmd.set_buffer(&res.instance_ssbo, 0..res.instance_ssbo.len(), SCENE_SET_ID, 1);
    }));
}
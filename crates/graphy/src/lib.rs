#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_unsafe)]

#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate const_cstr;

pub mod buffer;
//pub mod command;
pub mod device;
pub mod error;
pub mod gfx;
pub mod graph;
pub mod shader;
pub mod texture;

pub use device::Device;
pub use error::*;
pub use gfx::Gfx;

//pub use command::CommandBuffer;
//pub use graph::*;

pub use shader::*;
pub use texture::FilterMode;
pub use texture::Format;
pub use texture::Texture2D;
pub use texture::TextureConfig;
pub use texture::WrapMode;

pub use buffer::Buffer;
pub use buffer::ImmutableVertexBuffer;
pub use buffer::IndexBuffer;
pub use buffer::UniformBuffer;

mod swapchain;
use swapchain::Swapchain;

mod barrier;
mod descriptor;
mod util;

mod tests {
    use std::time::Instant;

    use ash::vk;
    use winit::{event::*, event_loop::*, window::*};

    // use crate::{
    //     graph::graphics::{pipeline::PipelineState, Pipeline},
    //     Gfx,
    // };

    use crate::{Gfx, descriptor};
    
    extern crate fern;
    use fern::*;


    const WIDTH: u32 = 800;
    const HEIGHT: u32 = 600;
    fn setup_logging() {
        let colors_line = fern::colors::ColoredLevelConfig::new()
            .error(fern::colors::Color::Red)
            .warn(fern::colors::Color::Yellow)
            .info(fern::colors::Color::Green)
            .debug(fern::colors::Color::BrightBlue)
            .trace(fern::colors::Color::BrightBlack);
        fern::Dispatch::new()
            .format(move |out, message, record| {
                out.finish(format_args!(
                    "{bold}{white}[{}]{reset} {}{s_reset} {}\n",
                    chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                    colors_line.color(record.level()),
                    message,
                    bold = crossterm::style::Attribute::Bold,
                    white = crossterm::style::SetForegroundColor(crossterm::style::Color::White),
                    reset = crossterm::style::ResetColor,
                    s_reset = crossterm::style::Attribute::Reset
                ))
            })
            .level(log::LevelFilter::Debug)
            .chain(std::io::stdout())
            .chain(fern::log_file("output.log").unwrap())
            .apply()
            .unwrap();
    }

    #[test]
    fn offscreen() -> Result<(), Box<dyn std::error::Error>> {
        setup_logging();

        let event_loop = EventLoop::<()>::new_any_thread();
        let mut window = WindowBuilder::new().build(&event_loop).unwrap();

        let mut gfx = Gfx::new(&mut window, true)?;

        let test_shader = crate::ShaderProgramBuilder::vertex_and_fragment(
            "test shader",
            &crate::ShaderCode {
                entry_point: "main".into(),
                data: crate::ShaderData::Glsl(include_str!("../shaders/pbr.vert").into()),
            },
            &crate::ShaderCode {
                entry_point: "main".into(),
                data: crate::ShaderData::Glsl(include_str!("../shaders/pbr.frag").into()),
            },
        )
        .build(gfx.device());

        let test_shader = match test_shader {
            Err(x) => {
                log::error!("{}", x);
                panic!();
            }
            Ok(x) => x,
        };

        let (data, width, height) =
            hikari_asset::image::load_from_file(std::path::Path::new(r"./dny1x058dj531.jpg"))?;

        let texture = crate::Texture2D::new(
            gfx.device(),
            &data,
            width,
            height,
            crate::TextureConfig {
                format: crate::texture::Format::RGBA8,
                wrap_x: crate::texture::WrapMode::Clamp,
                wrap_y: crate::texture::WrapMode::Clamp,
                filtering: crate::texture::FilterMode::Closest,
                aniso_level: 16,
                generate_mips: true,
            },
        )?;

        let texture = crate::texture::SampledImage::with_data(
            gfx.device(),
            &data,
            width,
            height,
            crate::texture::VkTextureConfig {
                format: vk::Format::R8G8B8A8_UNORM,
                filtering: vk::Filter::NEAREST,
                wrap_x: vk::SamplerAddressMode::CLAMP_TO_EDGE,
                wrap_y: vk::SamplerAddressMode::CLAMP_TO_EDGE,
                aniso_level: 0,
                mip_levels: 5,
                mip_filtering: vk::SamplerMipmapMode::NEAREST,
                aspect_flags: vk::ImageAspectFlags::COLOR,
                primary_image_layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
                host_readable: true,
                usage: vk::ImageUsageFlags::SAMPLED,
            },
        )?;

        // let now = std::time::Instant::now();

        // let pixels = unsafe { texture.read().unwrap() };

        // log::info!("{:?}", now.elapsed());

        // let mut ppm_file = format!("P3\n{} {}\n255\n", texture.width(), texture.height());

        // pixels.chunks(4).for_each(|colors| {
        //     let r = colors[0];
        //     let g = colors[1];
        //     let b = colors[2];
        //     ppm_file.push_str(&format!("{} {} {}\n", r, g, b));
        // });
        // std::fs::write("./test.ppm", ppm_file)?;

        // use crate::graph::graphics::pipeline::*;
        // use crate::graph::graphics::*;

        // let pipeline = Pipeline::new(
        //     test_shader,
        //     PipelineState {
        //         input_layout: Default::default(),
        //         primitive_topology: PrimitiveTopology::Triangles,
        //         rasterizer_info: RasterizerState {
        //             polygon_mode: PolygonMode::Line,
        //             ..Default::default()
        //         },
        //         depth_stencil_info: Default::default(),
        //         blend_info: Default::default(),
        //     },
        // );

        // let mut graph = crate::graph::GraphBuilder::<(), (), ()>::new()
        //     .add_renderpass(
        //         RenderpassBuilder::new(
        //             "Test",
        //             crate::graph::ImageSize::Relative(1.0, 1.0),
        //             |cmd, _, _, _| {
        //                 log::info!("Running Test");
        //             },
        //         )
        //         .color_output(
        //             "color_out",
        //             ColorOutput {
        //                 format: crate::graph::ColorFormat::R8G8B8A8_UNORM,
        //                 clear: true,
        //             },
        //         )
        //         .will_use_pipeline(&pipeline)
        //         .build()?,
        //     )
        //     .add_renderpass(
        //         RenderpassBuilder::new(
        //             "Test1",
        //             crate::graph::ImageSize::Relative(1.0, 1.0),
        //             |cmd, _, _, _| {
        //                 log::info!("Running Test1");
        //             },
        //         )
        //         .input("color_out")
        //         .color_output(
        //             "color_",
        //             ColorOutput {
        //                 format: crate::graph::ColorFormat::R8G8B8A8_UNORM,
        //                 clear: true,
        //             },
        //         )
        //         .depth_stencil_output(
        //             "depth_",
        //             DepthStencilOutput {
        //                 format: crate::graph::DepthStencilFormat::D16_UNORM,
        //                 depth_clear: true,
        //                 stencil_clear: true,
        //             },
        //         )
        //         .will_use_pipeline(&pipeline)
        //         .mark_final()
        //         .build()?,
        //     )
        //     .build(&gfx)?;

        Ok(())
        // event_loop.run(move |event, _, control_flow| {
        //     *control_flow = ControlFlow::Wait;
        //     log::debug!("Here");
        //     // graph
        //     //     .execute(&mut gfx, &(), &(), &())
        //     //     .expect("Graph execution failed");
        //     log::debug!("Post exec");

        //     match event {
        //         Event::WindowEvent {
        //             event: WindowEvent::CloseRequested,
        //             window_id,
        //         } if window_id == window.id() => *control_flow = ControlFlow::Exit,
        //         _ => (),
        //     }
        // })

        // println!("Going to wait...");
        // std::io::stdin().read_line(&mut String::new()).unwrap();
    }
}

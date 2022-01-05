#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_unsafe)]

pub mod error;
pub mod gltf;
pub mod image;
pub mod material;
pub mod mesh;
pub mod scene;
pub mod texture;

pub use error::Error;
pub use material::Material;
pub use mesh::Mesh;
pub use mesh::Model;
pub use scene::Scene;
pub use texture::Texture;

#[cfg(test)]
mod tests {
    //use itertools::izip;

    #[test]
    fn it_works() {
        // use glfw::Context;
        // let WIDTH = 800;
        // let HEIGHT = 600;
        // let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).expect("Failed to initialise GLFW");

        // glfw.window_hint(glfw::WindowHint::ContextVersion(4, 5));
        // glfw.window_hint(glfw::WindowHint::OpenGlProfile(
        //     glfw::OpenGlProfileHint::Core,
        // ));
        // glfw.window_hint(glfw::WindowHint::OpenGlDebugContext(true));
        // glfw.window_hint(glfw::WindowHint::OpenGlForwardCompat(true));

        // let (mut window, events) = glfw
        //     .create_window(
        //         WIDTH,
        //         HEIGHT,
        //         "OpenGL Template Rust",
        //         glfw::WindowMode::Windowed,
        //     )
        //     .expect("Failed to created GLFW window");

        // window.set_key_polling(true);
        // window.set_framebuffer_size_polling(true);

        // let gfx: graphy::Gfx = graphy::Gfx::new(&mut window, true).unwrap();

        // let now = std::time::Instant::now();
        // let crate::Scene {
        //     textures,
        //     models,
        //     materials,
        // } = crate::gltf::load_scene(std::path::Path::new("E:\\stressTest.glb"))
        //     .expect("Couldn't  load model");

        // println!("Loaded scene in {:?}", now.elapsed());

        // let now = std::time::Instant::now();

        // for texture in textures {
        //     graphy::texture::Texture2D::new(gfx.device(), texture.data(), texture.config())
        //         .unwrap();
        // }

        // struct Vertex {
        //     position: glam::Vec3A,
        //     normal: glam::Vec3A,
        //     tc0: glam::Vec2,
        //     tc1: glam::Vec2,
        // }

        // let mut modelData = Vec::new();
        // for model in models {
        //     for mesh in model.meshes() {
        //         let mut vertices = Vec::new();
        //         for (&position, &normal, &tc0, &tc1) in izip!(
        //             mesh.positions(),
        //             mesh.normals(),
        //             mesh.texcoord0(),
        //             mesh.texcoord1()
        //         ) {
        //             vertices.push(Vertex {
        //                 position,
        //                 normal,
        //                 tc0,
        //                 tc1,
        //             });
        //         }
        //         use graphy::ShaderDataType as dt;
        //         let vao = graphy::buffer::VertexArray::create()
        //             .vertexBuffer(
        //                 &graphy::buffer::ImmutableVertexBuffer::withData(
        //                     &vertices,
        //                     &[dt::Vec3f, dt::Vec3f, dt::Vec2f, dt::Vec2f],
        //                 )
        //                 .unwrap(),
        //             )
        //             .indexBuffer(&graphy::buffer::IndexBuffer::withData(mesh.indices()).unwrap())
        //             .build()
        //             .unwrap();

        //         modelData.push(vao);
        //     }
        // }

        // println!("Allocated resources in {:?}", now.elapsed());
    }
}

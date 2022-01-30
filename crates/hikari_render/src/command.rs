use std::ffi::c_void;

use crate::{buffer::VertexArray, texture::Texture, Buffer, Pipeline, UniformBuffer};
pub struct CommandBuffer {
    pipeline: Option<*const crate::graph::Pipeline>,
}

impl CommandBuffer {
    pub(crate) fn new() -> Self {
        Self { pipeline: None }
    }
    pub fn bind_vertex_array(&mut self, array: &VertexArray) {
        array.bind();
    }
    pub fn bind_pipeline(&mut self, pipeline: &crate::Pipeline) {
        // unsafe {
        //     log::debug!("Binding shader: {}", pipeline.shader().id());
        //     gl::UseProgram(pipeline.shader().id());
        //     self.pipeline.replace(pipeline as *const _);
        // }
    }
    fn pipeline(&self) -> &Pipeline {
        unsafe { &*self.pipeline.unwrap() }
    }
    pub fn set_texture<T: Texture>(&mut self, sampler_name: &str, texture: &T) {
        // let unit = self.pipeline().shader().reflection_data.uniforms()[sampler_name].binding;
        // unsafe {
        //     gl::ActiveTexture(gl::TEXTURE0 + unit);
        // }
        // texture.bind(unit);
    }
    pub fn update_uniform_buffer<T>(&mut self, buffer: &UniformBuffer<T>, data: &T) {
        //assert!(std::mem::size_of::<T>() == buffer.size());

        buffer.bind();

        let data_ptr = data as *const T;
        unsafe {
            gl::BufferSubData(
                UniformBuffer::<T>::buffer_target(),
                0,
                std::mem::size_of::<T>() as isize,
                data_ptr.cast(),
            );
        }
    }
    #[deprecated]
    pub fn set_int(&mut self, name: &str, data: i32) {
        panic!("Use push constants, updating uniforms individually has been deprecated")
    }
    #[deprecated]
    pub fn set_uint(&mut self, name: &str, data: u32) {
        panic!("Use push constants, updating uniforms individually has been deprecated")
    }
    #[deprecated]
    pub fn set_float(&mut self, name: &str, data: f32) {
        panic!("Use push constants, updating uniforms individually has been deprecated")
    }
    #[deprecated]
    pub fn set_vec2f(&mut self, name: &str, x: f32, y: f32) {
        panic!("Use push constants, updating uniforms individually has been deprecated")
    }
    #[deprecated]
    pub fn set_vec4f(&mut self, name: &str, x: f32, y: f32, z: f32, w: f32) {
        panic!("Use push constants, updating uniforms individually has been deprecated")
    }
    #[deprecated]
    pub fn set_mat4f(&mut self, name: &str, data: &glam::Mat4) {
        panic!("Use push constants, updating uniforms individually has been deprecated")
    }
    pub fn draw_indexed(&mut self, index_count: usize, offset: usize) {
        unsafe {
            gl::DrawElements(
                self.pipeline().bind_info().gl_topology,
                index_count as i32,
                gl::UNSIGNED_INT,
                offset as *const c_void,
            );
        }
    }
}

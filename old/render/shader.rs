use std::error::Error;

use crate::core::Handle;

use super::backend::{Device, GraphicsDevice};

pub struct Shader {
    inner: super::backend::Shader,
}
impl Shader {
    pub fn new<P: AsRef<str>>(
        context: &Handle<Device>,
        vertexPath: P,
        fragmentPath: P,
    ) -> Result<Self, Box<dyn Error>> {
        todo!()
        // Ok(Shader {
        //     inner: context.createShader(vertexPath, fragmentPath)?,
        // })
    }
    pub fn setActiveTexture(&mut self, textureUnit: u32) {
        self.inner.setActiveTexture(textureUnit);
    }
    pub fn setIntArr(&mut self, varName: &'static str, data: &[i32]) {
        self.inner.setIntArr(varName, data);
    }
    pub fn setFloat(&mut self, varName: &'static str, data: f32) {
        self.inner.setFloat(varName, data);
    }
    pub fn setVec3f(&mut self, varName: &'static str, data: &glm::Vec3) {
        self.inner.setVec3f(varName, data);
    }
    pub fn setVec4f(&mut self, varName: &'static str, data: &glm::Vec4) {
        self.inner.setVec4f(varName, data);
    }
    pub fn setMat4f(&mut self, varName: &'static str, data: &glm::Mat4) {
        self.inner.setMat4f(varName, data);
    }
}

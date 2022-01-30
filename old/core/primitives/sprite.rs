use crate::rendering::{};

use super::Arc;

#[derive(Clone)]
pub struct Sprite {
    tex: Arc<SubTexture2D>,
    aspect: f32,
}

impl Sprite {
    pub fn fromSubTexture(tex: &Arc<SubTexture2D>) -> Self {
        let tex = tex.clone();

        let aspect = tex.height() / tex.width();

        Self { tex, aspect }
    }
    pub fn fromTexture(tex: &Arc<Texture2D>) -> Self {
        let tex = SubTexture2D::new(&tex, &glm::vec2(0.0, 0.0), &glm::vec2(1.0, 1.0));

        let aspect = tex.height() / tex.width();
        Self { tex, aspect }
    }

    ///Accepts pixel coordinates
    pub fn fromTextureWithRegion(
        tex: &Arc<Texture2D>,
        topLeft: (u32, u32),
        dimensions: (u32, u32),
    ) -> Self {
        let topLeft = glm::vec2(
            topLeft.0 as f32 / tex.width() as f32,
            topLeft.1 as f32 / tex.height() as f32,
        );
        let dimensions = glm::vec2(
            dimensions.0 as f32 / tex.width() as f32,
            dimensions.1 as f32 / tex.height() as f32,
        );

        let tex = SubTexture2D::new(&tex, &topLeft, &dimensions);

        let aspect = dimensions.y / dimensions.x;
        Self { tex, aspect }
    }
    pub fn getTexture(&self) -> &Arc<SubTexture2D> {
        &self.tex
    }
    pub fn aspect(&self) -> f32 {
        self.aspect
    }
}

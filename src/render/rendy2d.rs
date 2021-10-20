//pub use shader::Shader;
struct TextureCounter {
    textureSlots: [u32; TEXTURE_UNITS],
    nUsed: usize,
}
impl TextureCounter {
    fn new() -> Self {
        const TEXTURE_SLOTS: [u32; TEXTURE_UNITS] = [0; TEXTURE_UNITS];
        Self {
            textureSlots: TEXTURE_SLOTS,
            nUsed: 0,
        }
    }
    fn reset(&mut self) {
        self.nUsed = 0;
    }
    fn getSlot(&mut self, internalID: u32) -> u32 {
        for i in 0..self.nUsed {
            let slot = self.textureSlots[i];
            if slot == internalID {
                return i as u32;
            }
        }

        self.textureSlots[self.nUsed] = internalID;
        let ret = self.nUsed as u32;
        self.nUsed += 1;

        ret
    }
    fn isSlotAvailable(&self) -> bool {
        self.nUsed < TEXTURE_UNITS
    }
}
//glm::vec2(..) is not a const function and vec2 is not guaranteed to be stored in C's memory representation, so I'm using a custom Vec2 struct
#[repr(C)]
#[derive(Clone, Copy, Debug)]
struct Vector2 {
    x: f32,
    y: f32,
}
#[repr(C)]
#[derive(Clone, Copy, Debug)]
struct Vertex {
    position: Vector2,
    texID: f32,
}
impl Vertex {
    const fn empty() -> Self {
        Self {
            position: Vector2 { x: 0.0, y: 0.0 },
            texID: 0.0,
        }
    }
    const fn new(x: f32, y: f32, texID: f32) -> Self {
        Self {
            position: Vector2 { x, y },
            texID,
        }
    }
}

static mut localData: [Vertex; MAX_QUADS * 4] = [Vertex::empty(); MAX_QUADS * 4];

pub(crate) mod backend;
pub mod camera;
mod shader;
pub mod texture;

pub use shader::Shader;
pub use texture::SubTexture2D;
pub use texture::Texture;
pub use texture::Texture2D;


struct BatchData {
    batchVertexBuffer: VertexBuffer,
    batchIndexBuffer: IndexBuffer,
    batchTexCoords: VertexBuffer,
    textureCounter: TextureCounter,
    texSlotWithID: Vec<i32>,

    quadCount: usize,
}

impl BatchData {
    fn generateIndices() -> [i32; MAX_QUADS * 6] {
        let mut indexBuffer = [0; MAX_QUADS * QUAD_INDS.len()];
        let mut i = 0;
        let mut offset = 0;

        while i < indexBuffer.len() {
            let x = &mut indexBuffer[i..i + 6];

            // x.clone_from_slice(&[QUAD_INDS[0] + offset,
            //                          QUAD_INDS[1] + offset]);

            x.clone_from_slice(&[
                QUAD_INDS[0] + offset,
                QUAD_INDS[1] + offset,
                QUAD_INDS[2] + offset,
                QUAD_INDS[3] + offset,
                QUAD_INDS[4] + offset,
                QUAD_INDS[5] + offset,
            ]);

            // indexBuffer[i + 0] = QUAD_INDS[0] + offset;
            // indexBuffer[i + 1] = QUAD_INDS[1] + offset;
            // indexBuffer[i + 2] = QUAD_INDS[2] + offset;
            // indexBuffer[i + 3] = QUAD_INDS[3] + offset;
            // indexBuffer[i + 4] = QUAD_INDS[4] + offset;
            // indexBuffer[i + 5] = QUAD_INDS[5] + offset;

            offset += 4;
            i += 6;
        }
        //assert!(false);
        indexBuffer
    }
    fn new(renderContext: &Handle<Device>) -> Self {
        // let batchVertices = renderContext.createVertexBuffer(MAX_QUADS * 4 * 3, &[]);
        // let batchIndexBuffer = renderContext.createIndexBuffer(&Self::generateIndices());

        // let textureCounter = TextureCounter::new();

        // let batchTexCoords =
        //     renderContext.createVertexBuffer(MAX_QUADS * 4 * 2, &QUAD_TEX_COORDS.repeat(MAX_QUADS));

        // use backend::DataType::*;

        // renderContext.setInputLayout(&[
        //     BufferLayout::new(&[("position", Vec2f), ("id", Vec1f)]),
        //     BufferLayout::new(&[("texCoords", Vec2f)]),
        // ]);
        // renderContext.setVertexBuffers(&[&batchVertices, &batchTexCoords]);

        // renderContext.setIndexBuffer(&batchIndexBuffer);

        let mut texSlotWithID = Vec::<i32>::new();

        (0..TEXTURE_UNITS).for_each(|i| {
            texSlotWithID.push(i as i32);
        });
        todo!();
        // Self {
        //     batchVertexBuffer: batchVertices,
        //     batchTexCoords,
        //     batchIndexBuffer,
        //     textureCounter,
        //     quadCount: 0,
        //     texSlotWithID,
        // }
    }
    fn reset(&mut self) {
        self.quadCount = 0;
        self.textureCounter.reset();
    }
}
const BATCH_QUAD: [Vertex; 4] = [
    Vertex::new(-1.0, 1.0, 0.0),
    Vertex::new(1.0, 1.0, 0.0),
    Vertex::new(1.0, -1.0, 0.0),
    Vertex::new(-1.0, -1.0, 0.0),
];

//TODO: refactor
fn glInit() {
    unsafe {
        gl::ClearColor(0.5, 0.5, 0.5, 1.0);
        //gl::ClearColor(0.0, 0.746, 1.0, 1.0);
        gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
        gl::Enable(gl::BLEND);
    }
}
pub struct Renderer2D {
    batchData: BatchData,
    camera: Camera2D,
    basicShader: backend::Shader,
    context: Handle<Device>,
}
impl Renderer2D {
    fn getTransformMatrix(transform: &Transform) -> glm::Mat4 {
        let mut transformMat = glm::translate(&glm::identity(), &transform.position);

        transformMat = glm::scale(&transformMat, &transform.scale);

        transformMat
    }
    pub fn new(renderContext: &Handle<Device>, viewportWidth: u32, viewportHeight: u32) -> Self {
        glInit();
        let renderContext = renderContext.clone();
        Self::resizeViewport(&renderContext, viewportWidth, viewportHeight);

        // let basicShader = renderContext
        //     .createShader(
        //         "Hikari//assets//shaders//vertex_batched.glsl",
        //         "Hikari//assets//shaders//texture.glsl",
        //     )
        //     .unwrap();

        // renderContext.setShader(&basicShader);

        // let camera = Camera2D::new(glm::zero(), viewportWidth, viewportHeight);

        // Self {
        //     batchData: BatchData::new(&renderContext),
        //     context: renderContext,
        //     basicShader,
        //     camera,
        // }
        todo!()
    }
    pub fn resizeViewport(context: &Handle<Device>, newWidth: u32, newHeight: u32) {
        context.resizeViewport(newWidth, newHeight);
    }
    pub fn onViewportResize(&mut self, newWidth: u32, newHeight: u32) {
        let currentPosition = *self.camera.getPosition();
        self.camera = Camera2D::new(currentPosition, newWidth, newHeight);

        Self::resizeViewport(&self.context, newWidth, newHeight);
    }
    pub fn setTarget(&mut self) {}

    pub fn getCamera<'a>(&'a mut self) -> &'a mut Camera2D {
        &mut self.camera
    }
    pub fn begin(&mut self) {
        self.basicShader
            .setMat4f("viewProjMat", self.camera.getViewProjectionMatrix());
    }
    fn drawTexturedQuad_int<T: Texture>(
        &mut self,
        position: &glm::Vec3,
        scale: &glm::Vec3,
        texture: &Handle<T>,
    ) {
        let mut batchData = &mut self.batchData;

        let texID = batchData.textureCounter.getSlot(texture.getRendererID());

        self.basicShader.setActiveTexture(texID);

        //self.context.setTexture2D(texture.raw());

        for (i, vert) in BATCH_QUAD.iter().enumerate() {
            let index = batchData.quadCount * 4 + i;
            let localQuad = unsafe { &mut localData[index] };

            // let mut transformMat: glm::Mat4 = glm::identity();
            // glm::translate(&mut transformMat, &transform.position);
            // glm::scale(&mut transformMat, &transform.scale);

            localQuad.position.x = position.x + vert.position.x * scale.x;
            localQuad.position.y = position.y + vert.position.y * scale.y;

            localQuad.texID = texID as f32;
        }
        batchData.quadCount += 1;

        if batchData.quadCount == MAX_QUADS || !batchData.textureCounter.isSlotAvailable() {
            self.end();
        }
    }
    pub fn drawTexturedQuad<T: Texture>(&mut self, transform: &Transform, texture: &Handle<T>) {
        self.drawTexturedQuad_int(&transform.position, &transform.scale, texture);
    }
    pub fn drawSprite(&mut self, transform: &Transform, sprite: &Sprite) {
        let scale = glm::vec3(transform.scale.x, transform.scale.y * sprite.aspect(), 1.0);

        self.drawTexturedQuad_int(&transform.position, &scale, sprite.getTexture());
    }
    fn flushBatch(&mut self) {
        let batchData = &self.batchData;

        //Create an array of f32 s from the vertex data
        let data: &[f32] = unsafe {
            std::slice::from_raw_parts(
                localData.as_ptr() as *const f32,
                batchData.quadCount * 4 * size_of::<Vertex>() / size_of::<f32>(),
            )
        };

        let maxTexUnit = self.batchData.textureCounter.nUsed;
        let texSlotWithID = &self.batchData.texSlotWithID[0..maxTexUnit];

        self.basicShader.setIntArr("textures", texSlotWithID);

        {
            profile_scope!("SetSubData");
            batchData.batchVertexBuffer.setSubData(0, data);
        }

        {
            profile_scope!("DrawElements");

            self.context.drawIndexed((batchData.quadCount * 6) as u32);
        }
    }
    pub fn end(&mut self) {
        if self.batchData.quadCount != 0 {
            self.flushBatch();
        }
        self.batchData.reset();
    }

    pub fn render(&mut self, world: &legion::World) {
        profile_scope!("Render");

        self.context.clear();

        use legion::*;

        let mut query = <(&Transform, &Sprite)>::query();

        self.begin();

        for (transform, sprite) in query.iter(world) {
            self.drawSprite(transform, sprite);
        }

        self.end();
    }
}

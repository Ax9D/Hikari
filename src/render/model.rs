use std::{path::Path, sync::Arc};

use itertools::izip;

use crate::core::primitives::Entity;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct Vertex {
    position: glam::Vec3,
    normal: glam::Vec3,
    texcoord0: glam::Vec2,
    texcoord1: glam::Vec2,
}
pub struct MeshBuilder<'a> {
    positions: &'a [glam::Vec3],
    normals: Option<&'a [glam::Vec3]>,
    texcoord0: Option<&'a [glam::Vec2]>,
    texcoord1: Option<&'a [glam::Vec2]>,
    indices: Option<&'a [u32]>,
}

impl<'a> MeshBuilder<'a> {
    pub fn new(positions: &'a [glam::Vec3]) -> Self {
        Self {
            positions,
            indices: None,
            normals: None,
            texcoord0: None,
            texcoord1: None,
        }
    }
    pub fn normals(mut self, data: &'a [glam::Vec3]) -> Self {
        self.normals.replace(data);

        self
    }
    pub fn indices(mut self, data: &'a [u32]) -> Self {
        self.indices.replace(data);

        self
    }
    pub fn texcoord0(mut self, data: &'a [glam::Vec2]) -> Self {
        self.texcoord0.replace(data);

        self
    }
    pub fn texcoord1(mut self, data: &'a [glam::Vec2]) -> Self {
        self.texcoord1.replace(data);

        self
    }
    pub fn build(
        self,
        material: &hikari_asset::Asset<super::Material>,
    ) -> Result<Mesh, Box<dyn std::error::Error>> {
        let default_normals;
        let normals = match self.normals {
            Some(normals) => normals,
            None => {
                default_normals = hikari_asset::mesh::default_normals(self.positions.len());
                default_normals.as_slice()
            }
        };

        let default_texcoord0;
        let texcoord0 = match self.texcoord0 {
            Some(texcoord0) => texcoord0,
            None => {
                default_texcoord0 = vec![glam::Vec2::ZERO; self.positions.len()];
                default_texcoord0.as_slice()
            }
        };

        let default_texcoord1;
        let texcoord1 = match self.texcoord1 {
            Some(texcoord1) => texcoord1,
            None => {
                default_texcoord1 = vec![glam::Vec2::ZERO; self.positions.len()];
                default_texcoord1.as_slice()
            }
        };

        let vertices = izip!(self.positions, normals, texcoord0, texcoord1)
            .map(|(position, normal, texcoord0, texcoord1)| Vertex {
                position: position.clone().into(),
                normal: normal.clone().into(),
                texcoord0: texcoord0.clone().into(),
                texcoord1: texcoord1.clone().into(),
            })
            .collect::<Vec<Vertex>>();

        let default_indices;
        let indices = match self.indices {
            Some(indices) => indices,
            None => {
                default_indices = (0..self.positions.len() as u32).collect::<Vec<_>>();

                default_indices.as_slice()
            }
        };

        //Mesh::fromRaw(&self.positions, &normals, &texcoord0, &texcoord1, &indices, &material)
        Mesh::from_vertices(&vertices, &indices, material)
    }
}
pub struct Mesh {
    pub(crate) vertex_array: Arc<graphy::buffer::VertexArray>,
    pub(crate) n_indices: usize,
    pub(crate) material: hikari_asset::Asset<super::Material>,
}
impl Mesh {
    pub fn from_vertices(
        vertices: &Vec<Vertex>,
        indices: &[u32],
        material: &hikari_asset::Asset<super::Material>,
    ) -> Result<Mesh, Box<dyn std::error::Error>> {
        use graphy::ShaderDataType as dt;

        let vertex_array = graphy::buffer::VertexArrayBuilder::new()
            .vertex_buffer(&graphy::buffer::ImmutableVertexBuffer::with_data(
                &vertices,
                &[dt::Vec3f, dt::Vec3f, dt::Vec2f, dt::Vec2f],
            )?)
            .index_buffer(&graphy::buffer::IndexBuffer::with_data(indices)?)
            .build()?;

        Ok(Mesh {
            vertex_array,
            material: material.clone(),
            n_indices: indices.len(),
        })
    }
    pub fn from_raw(
        positions: &[glam::Vec3],
        normals: &[glam::Vec3],
        tc0: &[glam::Vec2],
        tc1: &[glam::Vec2],
        indices: &[u32],
        material: &hikari_asset::Asset<super::Material>,
    ) -> Result<Mesh, Box<dyn std::error::Error>> {
        use graphy::ShaderDataType as dt;

        let vertex_array = graphy::buffer::VertexArrayBuilder::new()
            .vertex_buffer(&graphy::buffer::ImmutableVertexBuffer::with_data(
                positions,
                &[dt::Vec3f],
            )?)
            .vertex_buffer(&graphy::buffer::ImmutableVertexBuffer::with_data(
                normals,
                &[dt::Vec3f],
            )?)
            .vertex_buffer(&graphy::buffer::ImmutableVertexBuffer::with_data(
                tc0,
                &[dt::Vec2f],
            )?)
            .vertex_buffer(&graphy::buffer::ImmutableVertexBuffer::with_data(
                tc1,
                &[dt::Vec2f],
            )?)
            .index_buffer(&graphy::buffer::IndexBuffer::with_data(indices)?)
            .build()?;

        Ok(Mesh {
            vertex_array,
            material: material.clone(),
            n_indices: indices.len(),
        })
    }
}
pub struct Model {
    meshes: Vec<Mesh>,
}
impl Model {
    pub fn new(meshes: Vec<Mesh>) -> Self {
        Self { meshes }
    }
    pub fn meshes(&self) -> std::slice::Iter<'_, Mesh> {
        self.meshes.iter()
    }
}
pub struct ImportData {
    textures: Vec<hikari_asset::Asset<graphy::Texture2D>>,
    materials: Vec<hikari_asset::Asset<super::Material>>,
    models: Vec<hikari_asset::Asset<super::Model>>,
}
pub struct MeshComponent {
    pub(crate) model: hikari_asset::Asset<super::Model>,
}
pub fn spawn<P: AsRef<Path>>(
    ctx: &mut crate::Context,
    path: P,
    scene_p: &mut crate::Scene,
    transform: crate::render::Transform,
) -> Result<Entity, Box<dyn std::error::Error>> {
    let path = path.as_ref();
    let scene = hikari_asset::Scene::load(path)?;

    let mut textures = Vec::new();

    for texture in scene.textures() {
        let mut config = texture.config().clone();
        config.aniso_level = 4;
        textures.push(hikari_asset::Asset::new(
            texture.name(),
            path,
            graphy::Texture2D::new(ctx.gfx().device(), texture.data(), config)?,
        ))
    }

    let materials: Vec<_> = scene
        .materials()
        .map(|material| {
            let albedo = if let Some(albedo_desc) = material.albedo_map() {
                super::material::MaterialColor::Texture(
                    textures[albedo_desc.index()].clone(),
                    albedo_desc.tex_coord_set(),
                )
            } else {
                super::material::MaterialColor::Constant(*material.albedo())
            };

            let roughness = if let Some(roughness_desc) = material.roughness_map() {
                super::material::MaterialValue::Texture(
                    textures[roughness_desc.index()].clone(),
                    roughness_desc.tex_coord_set(),
                )
            } else {
                super::material::MaterialValue::Constant(material.roughness())
            };

            let metallic = if let Some(metallic_desc) = material.metallic_map() {
                super::material::MaterialValue::Texture(
                    textures[metallic_desc.index()].clone(),
                    metallic_desc.tex_coord_set(),
                )
            } else {
                super::material::MaterialValue::Constant(material.metallic())
            };

            let normal = if let Some(normal_desc) = material.normal_map() {
                super::material::MaterialValue::Texture(
                    textures[normal_desc.index()].clone(),
                    normal_desc.tex_coord_set(),
                )
            } else {
                super::material::MaterialValue::Constant(1.0)
            };

            let name = material.name();
            let material = super::Material {
                albedo,
                roughness,
                metallic,
                normal,
            };

            hikari_asset::Asset::new(name, path, material)
        })
        .collect();

    let mut models = Vec::new();
    for model in scene.models() {
        let mut meshes = Vec::new();

        for mesh in model.meshes() {
            let material = if let Some(material_ix) = mesh.material() {
                materials[material_ix].clone()
            } else {
                hikari_asset::Asset::new(
                    "Default Material",
                    std::path::Path::new(""),
                    super::Material::default(),
                )
            };

            meshes.push(
                MeshBuilder::new(mesh.positions())
                    .normals(mesh.normals())
                    .texcoord0(mesh.texcoord0())
                    .texcoord1(mesh.texcoord1())
                    .indices(mesh.indices())
                    .build(&material)?,
            );
        }
        let model = hikari_asset::Asset::new(model.name(), path, super::Model::new(meshes));
        models.push(model);
    }

    //TODO: Fix this shit
    let model = models.first().unwrap();
    let entity = scene_p.create_entity_with_transform(hikari_asset::Asset::name(model), transform);
    scene_p.add_component(
        entity,
        MeshComponent {
            model: model.clone(),
        },
    );

    Ok(entity)
}

use super::{Identifier, ResourceLoader};
use hashbrown::HashMap;
use rayon::prelude::*;
use std::sync::Arc;
use std::{
    fs,
    io::{BufReader, Cursor},
};
use steamengine_renderer::Renderer;
use wgpu::BufferUsages;
use wgpu::util::DrawIndexedIndirectArgs;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    position: [f32; 3],
    tex_coords: [f32; 2],
    normal: [f32; 3],
}
impl steamengine_renderer::vertex::Vertex for Vertex {
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 5]>() as wgpu::BufferAddress,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x3,
                },
            ],
        }
    }
}

pub struct RawModel {
    vertices: Vec<Vertex>,
    indices: Vec<u32>,
}
pub struct Models {
    pub vertices: wgpu::Buffer,
    pub indices: wgpu::Buffer,
}
pub trait RenderPassAttachModels {
    fn set_models(&mut self, models: &Models);
}
impl RenderPassAttachModels for wgpu::RenderPass<'_> {
    fn set_models(&mut self, models: &Models) {
        self.set_vertex_buffer(0, models.vertices.slice(..));
        self.set_index_buffer(models.indices.slice(..), wgpu::IndexFormat::Uint32);
    }
}

pub struct ModelResourceLoader;

impl ModelResourceLoader {
    pub fn new() -> Self {
        Self
    }
    pub fn load_to_buffers(
        &self,
        root: &str,
        renderer: Arc<Renderer>,
    ) -> Result<(Models, HashMap<Identifier, Vec<DrawIndexedIndirectArgs>>), crate::errors::Error>
    {
        let entities = self.load_all(root)?;

        let mut vertices: Vec<Vertex> = Vec::new();
        let mut indices: Vec<u32> = Vec::new();
        let mut commands = HashMap::new();

        for (id, meshes) in entities {
            for mesh in meshes {
                let mesh_vertices = mesh.vertices;
                let mesh_indices = mesh.indices;

                let base_vertex = vertices.len() as i32;
                let first_index = indices.len() as u32;
                let index_count = mesh_indices.len() as u32;

                vertices.extend(mesh_vertices);
                indices.extend(mesh_indices);

                commands
                    .entry(id.clone())
                    .or_insert(Vec::new())
                    .push(DrawIndexedIndirectArgs {
                        base_vertex,
                        index_count,
                        first_index,
                        instance_count: 0,
                        first_instance: 0,
                    });
            }
        }
        let vertices: &[u8] = bytemuck::cast_slice(&vertices);
        let indices: &[u8] = bytemuck::cast_slice(&indices);

        let vertices = renderer.init_buffer(
            "Vertex buffer",
            BufferUsages::VERTEX | BufferUsages::COPY_DST,
            vertices,
        );
        let indices = renderer.init_buffer(
            "Index buffer",
            BufferUsages::INDEX | BufferUsages::COPY_DST,
            indices,
        );
        let models = Models { vertices, indices };
        Ok((models, commands))
    }
}

impl ResourceLoader for ModelResourceLoader {
    type Resource = Vec<RawModel>;
    type Error = crate::errors::Error;
    fn label(&self) -> &'static str {
        "Model Resource Loader"
    }

    fn load_from_bytes(&self, bytes: Vec<u8>) -> Result<Self::Resource, Self::Error> {
        let text = String::from_utf8(bytes)?;
        let cursor = Cursor::new(text);
        let mut reader = BufReader::new(cursor);

        let (models, _materials) = tobj::load_obj_buf(
            &mut reader,
            &tobj::LoadOptions {
                triangulate: true,
                single_index: true,
                ..Default::default()
            },
            move |p| {
                let mat_text = fs::read_to_string(&p).expect("Cannot read to string file");
                tobj::load_mtl_buf(&mut BufReader::new(Cursor::new(mat_text)))
            },
        )?;

        let meshes: Vec<RawModel> = models
            .par_iter()
            .map(|m| {
                let vertices = (0..m.mesh.positions.len() / 3)
                    .into_par_iter()
                    .map(|i| {
                        if m.mesh.normals.is_empty() {
                            Vertex {
                                position: [
                                    m.mesh.positions[i * 3],
                                    m.mesh.positions[i * 3 + 1],
                                    m.mesh.positions[i * 3 + 2],
                                ],
                                tex_coords: [
                                    m.mesh.texcoords[i * 2],
                                    1.0 - m.mesh.texcoords[i * 2 + 1],
                                ],
                                normal: [0.0, 0.0, 0.0],
                            }
                        } else {
                            Vertex {
                                position: [
                                    m.mesh.positions[i * 3],
                                    m.mesh.positions[i * 3 + 1],
                                    m.mesh.positions[i * 3 + 2],
                                ],
                                tex_coords: [
                                    m.mesh.texcoords[i * 2],
                                    1.0 - m.mesh.texcoords[i * 2 + 1],
                                ],
                                normal: [
                                    m.mesh.normals[i * 3],
                                    m.mesh.normals[i * 3 + 1],
                                    m.mesh.normals[i * 3 + 2],
                                ],
                            }
                        }
                    })
                    .collect::<Vec<_>>();

                let indices = m.mesh.indices.clone();

                RawModel { vertices, indices }
            })
            .collect::<Vec<_>>();

        Ok(meshes)
    }
}

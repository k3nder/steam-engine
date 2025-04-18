use wgpu::VertexBufferLayout;

use crate::render::render_pipeline::RenderPipeline;

pub trait Vertex: Copy + Clone + bytemuck::Pod + bytemuck::Zeroable {
    fn desc() -> VertexBufferLayout<'static>;
}

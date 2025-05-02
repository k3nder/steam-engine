use wgpu::VertexBufferLayout;

pub trait Vertex: Copy + Clone + bytemuck::Pod + bytemuck::Zeroable {
    fn desc() -> VertexBufferLayout<'static>;
}

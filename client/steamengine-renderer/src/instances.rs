use wgpu::VertexBufferLayout;

pub trait Instance<A: RawInstance> {
    fn to_raw(&self) -> A;
}
pub trait RawInstance: Copy + Clone + bytemuck::Pod + bytemuck::Zeroable {
    fn desc() -> VertexBufferLayout<'static>;
}

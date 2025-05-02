use wgpu::VertexBufferLayout;
/// This trait is the layout of one vertex
/// ## Example
/// ```rust
///
/// // The vertex has a position and color values
/// #[repr(C)]
/// #[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
/// pub struct Vertex3DColor {
///     pub position: [f32; 3],
///     pub color: [f32; 3],
/// }
/// impl Vertex for Vertex3DColor {
/// // This is the description of the vertex
///
///   const ATTRIBS: [wgpu::VertexAttribute; 2] =
///      wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x3];
///
///     fn desc() -> wgpu::VertexBufferLayout<'static> {
///         use std::mem;
///
///        wgpu::VertexBufferLayout {
///            array_stride: mem::size_of::<Self>() as wgpu::BufferAddress,
///            step_mode: wgpu::VertexStepMode::Vertex,
///            attributes: &Self::ATTRIBS,
///        }
///     }
/// }
/// ```
pub trait Vertex: Copy + Clone + bytemuck::Pod + bytemuck::Zeroable {
    fn desc() -> VertexBufferLayout<'static>;
}

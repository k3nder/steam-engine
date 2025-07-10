use cgmath::*;
use std::mem::size_of;
use std::sync::Arc;
use steamengine_renderer::Renderer;
use steamengine_renderer::vertex::Vertex;
use steamengine_renderer_util::simple_buffer::SimpleBuffer;
use tracing::*;
use wgpu::Buffer;
use wgpu::BufferAddress;
use wgpu::BufferUsages;
use wgpu::VertexAttribute;
use wgpu::VertexStepMode;

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable, Debug)]
pub struct RawInstance {
    matrix: [[f32; 4]; 4],
    color: [f32; 4],
    uv_offset: [f32; 2],
    uv_scale: [f32; 2],
}
const INSTANCE_ATTRIBS: [VertexAttribute; 7] = wgpu::vertex_attr_array![
    // Matrix
    5 => Float32x4,
    6 => Float32x4,
    7 => Float32x4,
    8 => Float32x4,
    // Color
    9 => Float32x4,
    // UV
    10 => Float32x2,
    11 => Float32x2,
];
impl Vertex for RawInstance {
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: (std::mem::size_of::<Self>()) as BufferAddress,
            step_mode: VertexStepMode::Instance,
            attributes: &INSTANCE_ATTRIBS,
        }
    }
}
#[derive(Clone)]
pub struct Instance {
    matrix: Matrix4<f32>,
    color: Vector4<f32>,
    uv_offset: [f32; 2],
    uv_scale: [f32; 2],
}
impl Instance {
    pub fn to_raw(self) -> RawInstance {
        let color: [f32; 4] = self.color.into();
        let matrix: [[f32; 4]; 4] = self.matrix.into();
        RawInstance {
            color,
            matrix,
            uv_scale: self.uv_scale,
            uv_offset: self.uv_offset,
        }
    }
}
impl Instance {
    pub fn new(
        color: crate::color::Color,
        matrix: Matrix4<f32>,
        texture: steamengine_renderer_util::resources::texture::TextureBounds,
    ) -> Self {
        Self {
            color,
            matrix,
            uv_offset: texture.uv_offset,
            uv_scale: texture.uv_scale,
        }
    }
}

pub struct InstanceBuffer<'a> {
    buffer: Buffer,
    renderer: Arc<Renderer<'a>>,
    limit: u64,
}
impl<'a> SimpleBuffer<'a, RawInstance> for InstanceBuffer<'a> {
    fn new(renderer: Arc<Renderer<'a>>, limit: u64) -> Self {
        let lock = renderer.clone();
        let buffer = lock.create_buffer(
            "Instance Buffer",
            BufferUsages::VERTEX | BufferUsages::COPY_DST,
            size_of::<RawInstance>() as u64 * limit,
        );
        Self {
            renderer,
            buffer,
            limit,
        }
    }
    fn as_entrie(&self) -> wgpu::BindingResource {
        self.buffer.as_entire_binding()
    }
    fn buffer(&self) -> &wgpu::Buffer {
        &self.buffer
    }
    fn renderer(&self) -> Arc<Renderer<'a>> {
        self.renderer.clone()
    }
    fn limit(&self) -> u64 {
        self.limit
    }
}

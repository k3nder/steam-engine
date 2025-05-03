use bytemuck::{Pod, Zeroable};
use steamengine_core::wgpu::{self, BufferAddress, VertexAttribute};

const ATTRIBS: [VertexAttribute; 1] = wgpu::vertex_attr_array![0 => Float32x3];

pub const VERTICES: [Vertex; 3] = [
    Vertex {
        position: [0.0, 0.5, 0.0],
    },
    Vertex {
        position: [-0.5, -0.5, 0.0],
    },
    Vertex {
        position: [0.5, -0.5, 0.0],
    },
];

pub const INDICES: [u16; 3] = [0, 1, 2];

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
}

impl steamengine_core::render::vertex::Vertex for Vertex {
    fn desc() -> steamengine_core::wgpu::VertexBufferLayout<'static> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Self>() as BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &ATTRIBS,
        }
    }
}

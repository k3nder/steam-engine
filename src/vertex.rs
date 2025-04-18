use wgpu::VertexBufferLayout;

use crate::render::render_pipeline::RenderPipeline;

trait Vertex: Copy + Clone + bytemuck::Pod + bytemuck::Zeroable {
    fn desc() -> VertexBufferLayout<'static>;
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex3DColor {
    pub position: [f32; 3],
    pub color: [f32; 3],
}
impl Vertex3DColor {
    fn new(position: [f32; 3], color: [f32; 3]) -> Self {
        Self { position, color }
    }
}
impl Vertex for Vertex3DColor {
    fn desc() -> VertexBufferLayout<'static> {
        VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x3,
                },
            ],
        }
    }
}

pub struct Vertex3DColorPipeline;

impl Vertex3DColorPipeline {
    pub fn new() -> Self {
        Self {}
    }
}

impl RenderPipeline for Vertex3DColorPipeline {
    fn label(&self) -> &str {
        "Vertex 3D color render pipeline"
    }

    fn source(&self) -> &str {
        r#"
        // Vertex shader

        struct VertexInput {
            @location(0) position: vec3<f32>,
            @location(1) color: vec3<f32>,
        };

        struct VertexOutput {
            @builtin(position) clip_position: vec4<f32>,
            @location(0) color: vec3<f32>,
        };

        @vertex
        fn vs_main(
            model: VertexInput,
        ) -> VertexOutput {
            var out: VertexOutput;
            out.color = model.color;
            out.clip_position = vec4<f32>(model.position, 1.0);
            return out;
        }

        // Fragment shader

        @fragment
        fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
            return vec4<f32>(in.color, 1.0);
        }

        "#
    }
    fn buffers(&self) -> Vec<VertexBufferLayout> {
        vec![Vertex3DColor::desc()]
    }
}

use wgpu::{BindGroupLayout, VertexBufferLayout};

use crate::render::vertex::{Vertex, VertexBasicWithTexture};

use super::RenderPipeline;

pub struct BasicRenderPipeline;
impl BasicRenderPipeline {
    pub fn new() -> Self {
        Self {}
    }
}
impl RenderPipeline for BasicRenderPipeline {
    fn label(&self) -> &str {
        "Basic Render Pipeline"
    }

    fn source(&self) -> &str {
        r#"
        // Vertex shader

        struct VertexOutput {
            @builtin(position) clip_position: vec4<f32>,
        };

        @vertex
        fn vs_main(
            @builtin(vertex_index) in_vertex_index: u32,
        ) -> VertexOutput {
            var out: VertexOutput;
            let x = f32(1 - i32(in_vertex_index)) * 0.5;
            let y = f32(i32(in_vertex_index & 1u) * 2 - 1) * 0.5;
            out.clip_position = vec4<f32>(x, y, 0.0, 1.0);
            return out;
        }

        @fragment
        fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
            return vec4<f32>(0.3, 0.2, 0.1, 1.0);
        }
        "#
    }
}

pub struct BasicTexturePipeline<'a> {
    bind_group_layouts: &'a [&'a BindGroupLayout],
}

impl<'a> BasicTexturePipeline<'a> {
    pub fn new(bind_group_layouts: &'a [&'a BindGroupLayout]) -> Self {
        Self { bind_group_layouts }
    }
}

impl<'a> RenderPipeline for BasicTexturePipeline<'a> {
    fn label(&self) -> &str {
        "Basic vertex texture pipeline"
    }

    fn source(&self) -> &str {
        r#"
        // Vertex shader

        struct VertexInput {
            @location(0) position: vec3<f32>,
            @location(1) tex_coords: vec2<f32>,
        }

        struct VertexOutput {
            @builtin(position) clip_position: vec4<f32>,
            @location(0) tex_coords: vec2<f32>,
        }

        @vertex
        fn vs_main(
            model: VertexInput,
        ) -> VertexOutput {
            var out: VertexOutput;
            out.tex_coords = model.tex_coords;
            out.clip_position = vec4<f32>(model.position, 1.0);
            return out;
        }

        // Fragment shader

        @group(0) @binding(0)
        var t_diffuse: texture_2d<f32>;
        @group(0) @binding(1)
        var s_diffuse: sampler;

        @fragment
        fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
            return textureSample(t_diffuse, s_diffuse, in.tex_coords);
        }
        "#
    }
    fn buffers(&self) -> Vec<VertexBufferLayout> {
        vec![VertexBasicWithTexture::desc()]
    }
    fn layout(&self) -> wgpu::PipelineLayoutDescriptor {
        wgpu::PipelineLayoutDescriptor {
            label: Some(self.label()),
            bind_group_layouts: self.bind_group_layouts,
            push_constant_ranges: &[],
        }
    }
}

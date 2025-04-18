use std::{fmt::Debug, num::NonZero};

use wgpu::{PipelineCache, PipelineCompilationOptions, RenderPass, VertexBufferLayout};

use super::Renderer;

pub mod basic;

pub trait RenderPipeline {
    fn label(&self) -> &str;
    fn source(&self) -> &str;
    fn buffers(&self) -> &[VertexBufferLayout] {
        &[]
    }
    fn vertex_compilation(&self) -> PipelineCompilationOptions {
        PipelineCompilationOptions::default()
    }
    fn targets<'a>(&self, renderer: &Renderer) -> Vec<Option<wgpu::ColorTargetState>> {
        let format = renderer.config.format.clone();
        vec![Some(wgpu::ColorTargetState {
            // 4.
            format,
            blend: Some(wgpu::BlendState::REPLACE),
            write_mask: wgpu::ColorWrites::ALL,
        })]
    }
    fn fragment_compilation(&self) -> PipelineCompilationOptions {
        PipelineCompilationOptions::default()
    }
    fn primitive(&self) -> wgpu::PrimitiveState {
        wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList, // 1.
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw, // 2.
            cull_mode: Some(wgpu::Face::Back),
            // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
            polygon_mode: wgpu::PolygonMode::Fill,
            // Requires Features::DEPTH_CLIP_CONTROL
            unclipped_depth: false,
            // Requires Features::CONSERVATIVE_RASTERIZATION
            conservative: false,
        }
    }
    fn depth_stencil(&self) -> Option<wgpu::DepthStencilState> {
        None
    }
    fn multisample(&self) -> wgpu::MultisampleState {
        wgpu::MultisampleState {
            count: 1,
            mask: !0,
            alpha_to_coverage_enabled: false,
        }
    }
    fn multiview(&self) -> Option<NonZero<u32>> {
        None
    }
    fn cache(&self) -> Option<&PipelineCache> {
        None
    }
    fn layout(&self) -> wgpu::PipelineLayoutDescriptor {
        wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[],
            push_constant_ranges: &[],
        }
    }
    fn to_wgpu<'a>(&self, renderer: &Renderer) -> wgpu::RenderPipeline {
        let shader = renderer
            .device()
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some(format!("Shader Source of {}", self.label()).as_str()),
                source: wgpu::ShaderSource::Wgsl(self.source().into()),
            });
        let layout = renderer.device().create_pipeline_layout(&self.layout());

        let render_pipeline =
            renderer
                .device()
                .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: Some(format!("Render Pipeline of {}", self.label()).as_str()),
                    layout: Some(&layout),
                    vertex: wgpu::VertexState {
                        module: &shader,
                        entry_point: Some("vs_main"),
                        buffers: self.buffers(),
                        compilation_options: self.vertex_compilation(),
                    },
                    fragment: Some(wgpu::FragmentState {
                        module: &shader,
                        entry_point: Some("fs_main"),
                        targets: self.targets(renderer).as_slice(),
                        compilation_options: self.fragment_compilation(),
                    }),
                    primitive: self.primitive(),
                    depth_stencil: self.depth_stencil(),
                    multisample: self.multisample(),
                    multiview: self.multiview(),
                    cache: self.cache(),
                });

        render_pipeline
    }
}

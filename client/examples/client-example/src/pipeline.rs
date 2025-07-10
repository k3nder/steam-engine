use crate::buffers::RawInstance;
use steamengine_renderer::render_pipeline::RenderPipeline;
use steamengine_renderer::vertex::Vertex;
use steamengine_renderer_util::depth_texture::DefaultDepthTexture;
use steamengine_renderer_util::depth_texture::DepthTexture;
use wgpu::BindGroupLayout;
use wgpu::PipelineLayoutDescriptor;

pub struct AppRenderPipeline<'a> {
    bind_group_layouts: &'a [&'a BindGroupLayout],
}
impl<'a> AppRenderPipeline<'a> {
    pub fn new(bind_group_layouts: &'a [&'a BindGroupLayout]) -> Self {
        AppRenderPipeline { bind_group_layouts }
    }
}
impl<'a> RenderPipeline for AppRenderPipeline<'a> {
    fn label(&self) -> &str {
        "Render pipeline"
    }

    fn source(&self) -> &str {
        include_str!("shader.wgsl")
    }

    fn buffers(&self) -> Vec<wgpu::VertexBufferLayout> {
        vec![
            steamengine_renderer_util::resources::model::Vertex::desc(),
            RawInstance::desc(),
        ]
    }
    fn layout(&self) -> wgpu::PipelineLayoutDescriptor {
        PipelineLayoutDescriptor {
            label: Some("Render pipeline layout"),
            bind_group_layouts: self.bind_group_layouts,
            push_constant_ranges: &[],
        }
    }
    fn depth_stencil(&self) -> Option<wgpu::DepthStencilState> {
        Some(DefaultDepthTexture::pipeline_stencil())
    }
}

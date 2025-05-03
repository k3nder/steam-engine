use steamengine_core::render::{instances::RawInstance, vertex::Vertex};

use crate::instances::InstanceRaw;

pub struct RenderPipeline;
impl RenderPipeline {
    pub fn new() -> Self {
        RenderPipeline
    }
}
impl steamengine_core::render::render_pipeline::RenderPipeline for RenderPipeline {
    fn label(&self) -> &str {
        "Render pipeline"
    }

    fn source(&self) -> &str {
        include_str!("./shader.wgsl")
    }

    fn buffers(&self) -> Vec<steamengine_core::wgpu::VertexBufferLayout> {
        vec![crate::vertex::Vertex::desc(), InstanceRaw::desc()]
    }
}

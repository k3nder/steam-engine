use wgpu::{
    QuerySet, RenderPassColorAttachment, RenderPassDepthStencilAttachment, RenderPassDescriptor,
    RenderPassTimestampWrites, TextureView,
};
pub struct RenderPassDescriptorBuilder<'a> {
    label: &'a str,
    color_attachments: &'a [Option<RenderPassColorAttachment<'a>>],
    depth_stencil_attachment: Option<RenderPassDepthStencilAttachment<'a>>,
    occlusion_query_set: Option<&'a QuerySet>,
    timestamp_writes: Option<RenderPassTimestampWrites<'a>>,
}
impl<'a> RenderPassDescriptorBuilder<'a> {
    pub fn new(label: &'a str) -> Self {
        Self {
            label,
            color_attachments: &[],
            depth_stencil_attachment: None,
            occlusion_query_set: None,
            timestamp_writes: None,
        }
    }

    pub fn with_colors(
        mut self,
        color_attachments: &'a [Option<RenderPassColorAttachment>],
    ) -> Self {
        self.color_attachments = color_attachments;
        self
    }
    pub fn with_depth(
        mut self,
        depth_stencil_attachment: RenderPassDepthStencilAttachment<'a>,
    ) -> Self {
        self.depth_stencil_attachment = Some(depth_stencil_attachment);
        self
    }
    pub fn with_occlusion(mut self, occlusion_query_set: &'a QuerySet) -> Self {
        self.occlusion_query_set = Some(occlusion_query_set);
        self
    }
    pub fn with_timestamp(mut self, timestamp_writes: RenderPassTimestampWrites<'a>) -> Self {
        self.timestamp_writes = Some(timestamp_writes);
        self
    }
    pub fn build(self) -> RenderPassDescriptor<'a> {
        RenderPassDescriptor {
            label: Some(self.label),
            color_attachments: self.color_attachments,
            depth_stencil_attachment: self.depth_stencil_attachment,
            timestamp_writes: self.timestamp_writes,
            occlusion_query_set: self.occlusion_query_set,
        }
    }
}

#[derive(Clone)]
pub struct RenderPassColorAttachmentBuilder<'a> {
    resolve_target: Option<&'a wgpu::TextureView>,
    ops: wgpu::Operations<wgpu::Color>,
}

impl<'a> RenderPassColorAttachmentBuilder<'a> {
    pub fn new() -> Self {
        Self {
            resolve_target: None,
            ops: wgpu::Operations::default(),
        }
    }
    pub fn resolve_target(mut self, resolve_target: &'a wgpu::TextureView) -> Self {
        self.resolve_target = Some(resolve_target);
        self
    }
    pub fn ops(mut self, ops: wgpu::Operations<wgpu::Color>) -> Self {
        self.ops = ops;
        self
    }
    pub fn from_color(r: f64, g: f64, b: f64, a: f64) -> Self {
        Self {
            resolve_target: None,
            ops: wgpu::Operations {
                load: wgpu::LoadOp::Clear(wgpu::Color { r, g, b, a }),
                store: wgpu::StoreOp::Store,
            },
        }
    }
    pub fn build(self, view: &'a wgpu::TextureView) -> RenderPassColorAttachment<'a> {
        RenderPassColorAttachment {
            view,
            resolve_target: self.resolve_target,
            ops: self.ops,
        }
    }
}

pub struct RenderPassDepthStencilAttachmentBuilder {
    depth_ops: Option<wgpu::Operations<f32>>,
    stencil_ops: Option<wgpu::Operations<u32>>,
}
impl RenderPassDepthStencilAttachmentBuilder {
    pub fn new() -> Self {
        Self {
            depth_ops: None,
            stencil_ops: None,
        }
    }
    pub fn depth_ops(mut self, ops: wgpu::Operations<f32>) -> Self {
        self.depth_ops = Some(ops);
        self
    }
    pub fn stencil_ops(mut self, ops: wgpu::Operations<u32>) -> Self {
        self.stencil_ops = Some(ops);
        self
    }
    pub fn build(self, view: &TextureView) -> RenderPassDepthStencilAttachment {
        RenderPassDepthStencilAttachment {
            view,
            depth_ops: self.depth_ops,
            stencil_ops: self.stencil_ops,
        }
    }
}

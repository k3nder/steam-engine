use wgpu::{RenderPassColorAttachment, RenderPassDepthStencilAttachment, TextureView};
#[macro_export]
macro_rules! color_render_pass {
    ($r:expr, $g:expr, $b:expr, $a:expr, $view:expr) => {
        steamengine_core::wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[Some(
                steamengine_core::render::render_pass::RenderPassColorAttachmentBuilder::from_color(
                    $r, $g, $b, $a,
                )
                .build(&$view),
            )],
            depth_stencil_attachment: None,
            occlusion_query_set: None,
            timestamp_writes: None,
        }
    };
    ($r:expr, $g:expr, $b:expr, $view:expr) => {
        steamengine_core::wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[Some(
                steamengine_core::render::render_pass::RenderPassColorAttachmentBuilder::from_color(
                    $r, $g, $b, 1.0,
                )
                .build(&$view),
            )],
            depth_stencil_attachment: None,
            occlusion_query_set: None,
            timestamp_writes: None,
        }
    };
    ($color:expr, $view:expr) => {
        steamengine_core::wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[Some(
                steamengine_core::render::render_pass::RenderPassColorAttachmentBuilder::from_color(
                    $color, $color, $color, 1.0,
                )
                .build(&$view),
            )],
            depth_stencil_attachment: None,
            occlusion_query_set: None,
            timestamp_writes: None,
        }
    };
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

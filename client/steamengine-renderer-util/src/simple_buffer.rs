use std::sync::Arc;
use steamengine_renderer::Renderer;
use tracing::*;
use wgpu::Buffer;
use wgpu::BufferUsages;
use wgpu::util::DrawIndexedIndirectArgs;

/// Abstraction of a buffer
pub trait SimpleBuffer<'a, T: bytemuck::NoUninit> {
    /// create a new buffer with limit of objects inside the buffer
    /// limit = size_of<T>() * limit;
    fn new(renderer: Arc<Renderer<'a>>, limit: u64) -> Self;
    /// Sets a unic entry inside the buffer
    fn set(&self, index: u64, data: T) {
        if index > self.limit() {
            error!(
                "attempt to nest an entity outside the limits of the buffer, SimpleBuffer Overflow"
            );
            return;
        }
        self.renderer()
            .update_buffer_entry(self.buffer(), index, data);
    }
    /// Sets all the buffer
    fn set_all(&self, data: &[T]) {
        if data.len() as u64 > self.limit() {
            error!(
                "attempt to nest an entity outside the limits of the buffer, SimpleBuffer Overflow"
            );
            return;
        }
        self.renderer().update_buffer(self.buffer(), data);
    }
    /// Converts the buffert to a binding resource
    fn as_entrie(&self) -> wgpu::BindingResource {
        self.buffer().as_entire_binding()
    }
    /// Gets the buffer
    fn buffer(&self) -> &wgpu::Buffer;
    fn renderer(&self) -> Arc<Renderer<'a>>;
    fn limit(&self) -> u64;
}

/// Implementation of simple buffer for commands buffer
pub struct DrawQueueBuffer<'a> {
    /// Wgpu Buffer
    buffer: Buffer,
    /// Renderer
    renderer: Arc<Renderer<'a>>,
    /// Limit of the buffer
    limit: u64,
}
impl<'a> SimpleBuffer<'a, DrawIndexedIndirectArgs> for DrawQueueBuffer<'a> {
    fn new(renderer: Arc<Renderer<'a>>, limit: u64) -> Self {
        let lock = renderer.clone();
        let buffer = lock.create_buffer(
            "Indexed Indirect Buffer",
            BufferUsages::INDIRECT | BufferUsages::COPY_DST,
            limit * std::mem::size_of::<DrawIndexedIndirectArgs>() as u64,
        );

        Self {
            buffer,
            renderer,
            limit,
        }
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

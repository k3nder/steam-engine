use std::sync::Arc;
use steamengine_renderer::Renderer;
use tracing::*;
use wgpu::Buffer;
use wgpu::BufferUsages;
use wgpu::util::DrawIndexedIndirectArgs;

/// Abstraction of a buffer
pub trait SimpleBuffer<'a, T> {
    /// create a new buffer with limit of objects inside the buffer
    /// limit = size_of<T>() * limit;
    fn new(renderer: Arc<Renderer<'a>>, limit: u64) -> Self;
    /// Sets a unic entry inside the buffer
    fn set(&self, index: u64, data: T);
    /// Sets all the buffer
    fn set_all(&self, data: &[T]);
    /// Converts the buffert to a binding resource
    fn as_entrie(&self) -> wgpu::BindingResource {
        self.buffer().as_entire_binding()
    }
    fn buffer(&self) -> &wgpu::Buffer;
}

pub struct DrawQueueBuffer<'a> {
    pub buffer: Buffer,
    renderer: Arc<Renderer<'a>>,
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
    fn set(&self, index: u64, data: DrawIndexedIndirectArgs) {
        let lock = self.renderer.clone();
        if index > self.limit {
            error!(
                "attempt to nest an entity outside the limits of the buffer, SimpleBuffer Overflow"
            );
            return;
        }
        lock.update_buffer_entry(&self.buffer, index, data);
    }
    fn set_all(&self, data: &[DrawIndexedIndirectArgs]) {
        let lock = self.renderer.clone();
        lock.update_buffer(&self.buffer, data);
    }
    fn as_entrie(&self) -> wgpu::BindingResource {
        self.buffer.as_entire_binding()
    }
    fn buffer(&self) -> &wgpu::Buffer {
        &self.buffer
    }
}

use std::sync::Arc;
use steamengine_renderer::Renderer;
use steamengine_renderer::bind_group::BindGroupEntryBuilder;

/// Bindings
pub struct Bindings {
    /// bind group of the bindings
    bind_group: Arc<wgpu::BindGroup>,
    /// layout of the bindings
    bind_group_layout: Arc<wgpu::BindGroupLayout>,
}
impl Bindings {
    /// gets the bind_group
    pub fn bind(&self) -> Arc<wgpu::BindGroup> {
        self.bind_group.clone()
    }
    /// gets the layout
    pub fn layout(&self) -> Arc<wgpu::BindGroupLayout> {
        self.bind_group_layout.clone()
    }
    /// create a new bind group
    pub fn new(bind_group: wgpu::BindGroup, bind_group_layout: wgpu::BindGroupLayout) -> Self {
        let bind_group = Arc::new(bind_group);
        let bind_group_layout = Arc::new(bind_group_layout);

        Self {
            bind_group,
            bind_group_layout,
        }
    }
}

pub trait CreateBindings {
    /// Create a new bindings with entries
    fn new_bindings(&self, label: &str, entries: &[BindGroupEntryBuilder]) -> Bindings;
}
impl CreateBindings for Renderer<'_> {
    fn new_bindings(&self, label: &str, entries: &[BindGroupEntryBuilder]) -> Bindings {
        let (bind, layout) = self.bind_group(label, entries);
        Bindings::new(bind, layout)
    }
}

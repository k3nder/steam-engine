use std::sync::Arc;
use steamengine_renderer::Renderer;
use steamengine_renderer::bind_group::BindGroupEntryBuilder;

pub struct Bindings {
    bind_group: Arc<wgpu::BindGroup>,
    bind_group_layout: Arc<wgpu::BindGroupLayout>,
}
impl Bindings {
    pub fn bind(&self) -> Arc<wgpu::BindGroup> {
        self.bind_group.clone()
    }
    pub fn layout(&self) -> Arc<wgpu::BindGroupLayout> {
        self.bind_group_layout.clone()
    }
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
    fn new_bindings(&self, label: &str, entries: &[BindGroupEntryBuilder]) -> Bindings;
}
impl CreateBindings for Renderer<'_> {
    fn new_bindings(&self, label: &str, entries: &[BindGroupEntryBuilder]) -> Bindings {
        let (bind, layout) = self.bind_group(label, entries);
        Bindings::new(bind, layout)
    }
}

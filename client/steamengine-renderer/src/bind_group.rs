use std::num::NonZero;

use wgpu::{BindingResource, BindingType, BufferBindingType, SamplerBindingType, ShaderStages};

/// this a builder for create a new bind group
pub struct BindGroupEntryBuilder<'a> {
    pub(crate) binding: u32,
    pub(crate) visibility: ShaderStages,
    pub(crate) ty: BindingType,
    pub(crate) count: Option<NonZero<u32>>,
    pub(crate) resource: Option<BindingResource<'a>>,
}

impl<'a> BindGroupEntryBuilder<'a> {
    /// create a new builder with the binding number
    pub fn new(binding: u32) -> Self {
        Self {
            binding,
            visibility: ShaderStages::FRAGMENT,
            ty: BindingType::Sampler(SamplerBindingType::Filtering),
            count: None,
            resource: None,
        }
    }
    /// sets the visibility of the bind group
    pub fn on(mut self, visibility: ShaderStages) -> Self {
        self.visibility = visibility;
        self
    }
    /// sets the type of the bind group
    pub fn of(mut self, ty: BindingType) -> Self {
        self.ty = ty;
        self
    }
    /// sets the content of the bind group
    pub fn with(mut self, resource: BindingResource<'a>) -> Self {
        self.resource = Some(resource);
        self
    }
    /// sets the count
    pub fn has(mut self, count: NonZero<u32>) -> Self {
        self.count = Some(count);
        self
    }
    pub fn uniform(self) -> Self {
        self.of(BindingType::Buffer {
            ty: BufferBindingType::Uniform,
            has_dynamic_offset: false,
            min_binding_size: None,
        })
    }
}

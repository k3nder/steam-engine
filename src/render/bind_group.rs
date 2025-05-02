use std::num::NonZero;

use wgpu::{BindingResource, BindingType, SamplerBindingType, ShaderStages};

pub struct BindGroupEntryBuilder<'a> {
    pub(crate) binding: u32,
    pub(crate) visibility: ShaderStages,
    pub(crate) ty: BindingType,
    pub(crate) count: Option<NonZero<u32>>,
    pub(crate) resource: Option<BindingResource<'a>>,
}

impl<'a> BindGroupEntryBuilder<'a> {
    pub fn new(binding: u32) -> Self {
        Self {
            binding,
            visibility: ShaderStages::FRAGMENT,
            ty: BindingType::Sampler(SamplerBindingType::Filtering),
            count: None,
            resource: None,
        }
    }
    pub fn on(mut self, visibility: ShaderStages) -> Self {
        self.visibility = visibility;
        self
    }
    pub fn of(mut self, ty: BindingType) -> Self {
        self.ty = ty;
        self
    }
    pub fn with(mut self, resource: BindingResource<'a>) -> Self {
        self.resource = Some(resource);
        self
    }
    pub fn has(mut self, count: NonZero<u32>) -> Self {
        self.count = Some(count);
        self
    }
}

use image::{ImageBuffer, Rgba};
use wgpu::{
    BindingType, Origin3d, Sampler, ShaderStages, TexelCopyBufferLayout, TexelCopyTextureInfo,
    TextureAspect, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages, TextureView,
    TextureViewDescriptor,
};

use super::{Renderer, bind_group::BindGroupEntryBuilder};

/// This structure contrains the dimensions of the texture
pub enum TextureDimensions {
    D3(u32, u32, u32),
    D2(u32, u32),
    D1(u32),
}
impl TextureDimensions {
    /// Create a new 3d texture dimension
    pub fn new_3d(width: u32, height: u32, depth: u32) -> Self {
        Self::D3(width, height, depth)
    }
    /// Create a new 2d texture dimension
    pub fn new_2d(width: u32, height: u32) -> Self {
        Self::D2(width, height)
    }
    /// Create a new 1d texture dimension
    pub fn new_1d(width: u32) -> Self {
        Self::D1(width)
    }

    /// Convert the dimension to wgpu::TextureDimension
    pub fn wgpu_texture_dimension(&self) -> TextureDimension {
        match self {
            Self::D3(_, _, _) => TextureDimension::D3,
            Self::D2(_, _) => TextureDimension::D2,
            Self::D1(_) => TextureDimension::D1,
        }
    }

    /// Build the dimension to an Extent3d
    pub fn build(self) -> wgpu::Extent3d {
        match self {
            Self::D3(width, height, depth) => wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: depth,
            },
            Self::D2(width, height) => wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            Self::D1(width) => wgpu::Extent3d {
                width,
                height: 1,
                depth_or_array_layers: 1,
            },
        }
    }
}
/// This is the builder of the texture
pub struct TextureBuilder {
    // Texture Size
    /// this is the size of the texture
    dimension: Option<TextureDimensions>,

    // Texture Descriptor
    /// default 1
    mip_level_count: Option<u32>,
    /// default 1
    sample_count: Option<u32>,
    /// default Bgra8UnormSrgb
    format: Option<TextureFormat>,
    /// default TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST
    usage: Option<TextureUsages>,

    // Texel info
    /// default 0
    mip_level: Option<u32>,
    /// default ZERO
    origin: Option<wgpu::Origin3d>,
    /// default All
    aspect: Option<wgpu::TextureAspect>,

    // buffer info
    /// default 0
    offset: Option<u64>,
    /// default 4 * size.width
    bytes_per_row: Option<u32>,
    /// default size.height
    rows_per_image: Option<u32>,

    // image data
    /// this is the data of the texture
    data: Option<ImageBuffer<Rgba<u8>, Vec<u8>>>,
}
impl TextureBuilder {
    /// create a new texture builder
    pub fn new() -> Self {
        Self {
            dimension: None,
            mip_level_count: None,
            sample_count: None,
            format: None,
            usage: None,
            mip_level: None,
            origin: None,
            aspect: None,
            offset: None,
            bytes_per_row: None,
            rows_per_image: None,
            data: None,
        }
    }

    pub fn mip_level_count(mut self, mip_level_count: u32) -> Self {
        self.mip_level_count = Some(mip_level_count);
        self
    }

    pub fn sample_count(mut self, sample_count: u32) -> Self {
        self.sample_count = Some(sample_count);
        self
    }

    pub fn dimension(mut self, dimension: TextureDimensions) -> Self {
        self.dimension = Some(dimension);
        self
    }

    pub fn format(mut self, format: TextureFormat) -> Self {
        self.format = Some(format);
        self
    }

    pub fn usage(mut self, usage: TextureUsages) -> Self {
        self.usage = Some(usage);
        self
    }

    pub fn mip_level(mut self, mip_level: u32) -> Self {
        self.mip_level = Some(mip_level);
        self
    }

    pub fn origin(mut self, origin: Origin3d) -> Self {
        self.origin = Some(origin);
        self
    }

    pub fn aspect(mut self, aspect: TextureAspect) -> Self {
        self.aspect = Some(aspect);
        self
    }

    pub fn buffer_offset(mut self, offset: u64) -> Self {
        self.offset = Some(offset);
        self
    }

    pub fn bytes_per_row(mut self, bytes_per_row: u32) -> Self {
        self.bytes_per_row = Some(bytes_per_row);
        self
    }

    pub fn rows_per_image(mut self, rows_per_image: u32) -> Self {
        self.rows_per_image = Some(rows_per_image);
        self
    }

    pub fn data(mut self, data: ImageBuffer<Rgba<u8>, Vec<u8>>) -> Self {
        self.data = Some(data);
        self
    }

    pub fn build(
        self,
        label: &'static str,
        view_formats: Option<&'static [TextureFormat]>,
        renderer: &Renderer,
    ) -> Texture {
        let dimensions = self.dimension.expect(
            "Unable to find texture dimensions on {}, please use function 'dimension' to set-it",
        );
        let dimension = dimensions.wgpu_texture_dimension();
        let size = dimensions.build();
        let mip_level_count = self.mip_level_count.unwrap_or(1);
        let sample_count = self.sample_count.unwrap_or(1);
        let format = self.format.unwrap_or(TextureFormat::Rgba8UnormSrgb);
        let usage = self
            .usage
            .unwrap_or(TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST);
        let label = Some(label);
        let data = self.data.expect("Image don't uploaded to texture");

        let texture = renderer.device().create_texture(&TextureDescriptor {
            size,
            mip_level_count,
            sample_count,
            dimension,
            format,
            usage,
            label,
            view_formats: view_formats.unwrap_or(&[]),
        });

        renderer.queue().write_texture(
            TexelCopyTextureInfo {
                texture: &texture,
                mip_level: self.mip_level.unwrap_or(0),
                origin: self.origin.unwrap_or(Origin3d::ZERO),
                aspect: self.aspect.unwrap_or(TextureAspect::All),
            },
            &data,
            TexelCopyBufferLayout {
                offset: self.offset.unwrap_or(0),
                bytes_per_row: self.bytes_per_row.or(Some(4 * size.width)),
                rows_per_image: self.rows_per_image.or(Some(size.height)),
            },
            size,
        );

        Texture {
            texture,
            texture_view: None,
            texture_sampler: None,
            bind_group: None,
            bind_group_layout: None,
        }
    }
}

pub struct Texture {
    pub texture: wgpu::Texture,
    pub texture_view: Option<TextureView>,
    pub texture_sampler: Option<Sampler>,
    pub bind_group: Option<wgpu::BindGroup>,
    pub bind_group_layout: Option<wgpu::BindGroupLayout>,
}
impl Texture {
    pub fn create_view(&self, descriptor: TextureViewDescriptor) -> TextureView {
        self.texture.create_view(&descriptor)
    }
    pub fn create_sampler(
        &self,
        descriptor: wgpu::SamplerDescriptor,
        renderer: &Renderer,
    ) -> Sampler {
        renderer.device().create_sampler(&descriptor)
    }
    pub fn texture_view(&mut self, descriptor: TextureViewDescriptor) {
        self.texture_view = Some(self.texture.create_view(&descriptor));
    }
    pub fn texture_sampler(&mut self, descriptor: wgpu::SamplerDescriptor, renderer: &Renderer) {
        self.texture_sampler = Some(self.create_sampler(descriptor, renderer));
    }
    pub fn default_bind_group(&mut self, label: &str, renderer: &Renderer) -> &wgpu::BindGroup {
        let texture_view = self
            .texture_view
            .as_ref()
            .expect("TextureView is None, use 'texture_view' function to set-it");
        let texture_sampler = self
            .texture_sampler
            .as_ref()
            .expect("TextureSampler is None, use 'texture_sampler' function to set-it");

        let (bind_group, bind_group_layout) = renderer.bind_group(
            label,
            &[
                BindGroupEntryBuilder::new(0)
                    .on(ShaderStages::FRAGMENT)
                    .of(BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    })
                    .with(wgpu::BindingResource::TextureView(texture_view)),
                BindGroupEntryBuilder::new(1)
                    .on(ShaderStages::FRAGMENT)
                    .of(BindingType::Sampler(wgpu::SamplerBindingType::Filtering))
                    .with(wgpu::BindingResource::Sampler(texture_sampler)),
            ],
        );

        self.bind_group = Some(bind_group);
        self.bind_group_layout = Some(bind_group_layout);

        self.bind_group.as_ref().unwrap()
    }
}

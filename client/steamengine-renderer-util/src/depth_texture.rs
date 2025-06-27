use steamengine_renderer::Renderer;
use steamengine_renderer::texture::Texture;
use steamengine_renderer::texture::TextureBuilder;
use steamengine_renderer::texture::TextureDimensions;
use wgpu::CompareFunction;
use wgpu::DepthBiasState;
use wgpu::DepthStencilState;
use wgpu::StencilState;
use wgpu::TextureFormat;

/// Abstraction of a depth texture
pub trait DepthTexture {
    /// Return the format of the texture, default Depth32Float
    fn format() -> TextureFormat {
        TextureFormat::Depth32Float
    }
    /// Return the compare funcion of the texture, default Less
    fn compare() -> CompareFunction {
        CompareFunction::Less
    }
    /// Return the stencil state, default Default::default
    fn stencil() -> StencilState {
        Default::default()
    }
    /// Return the bias, default Default::default
    fn bias() -> DepthBiasState {
        Default::default()
    }
    /// Return the pipeline config, default, with configurations
    fn pipeline_stencil() -> DepthStencilState {
        DepthStencilState {
            format: Self::format(),
            depth_write_enabled: true,
            depth_compare: Self::compare(),
            stencil: Self::stencil(),
            bias: Self::bias(),
        }
    }
    /// Create a new texture from renderer
    fn create_texture(renderer: &Renderer) -> Texture {
        let config = renderer.config();

        let mut texture = renderer.init_texture(
            "Depth texture",
            None,
            TextureBuilder::new()
                .format(Self::format())
                .dimension(TextureDimensions::new_2d(
                    config.width.max(1),
                    config.height.max(1),
                ))
                .usage(
                    wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
                ),
        );
        texture.texture_view(wgpu::TextureViewDescriptor::default());
        texture.texture_sampler(
            wgpu::wgt::SamplerDescriptor {
                label: Some("Depth sampler descriptor"),
                address_mode_u: wgpu::AddressMode::ClampToEdge,
                address_mode_v: wgpu::AddressMode::ClampToEdge,
                address_mode_w: wgpu::AddressMode::ClampToEdge,
                mag_filter: wgpu::FilterMode::Linear,
                min_filter: wgpu::FilterMode::Linear,
                mipmap_filter: wgpu::FilterMode::Nearest,
                lod_min_clamp: 0.0,
                lod_max_clamp: 100.0,
                compare: Some(Self::compare()),
                ..Default::default()
            },
            renderer,
        );

        texture
    }
    /// Create a new depth texture
    fn create(renderer: &Renderer) -> Self;
    ///
    fn stencil_attachment(&self) -> wgpu::RenderPassDepthStencilAttachment;
}

pub trait RenderPassCreateDepthTexture {
    fn create_depth_texture<T: DepthTexture>(&self) -> T;
}
impl RenderPassCreateDepthTexture for steamengine_renderer::Renderer<'_> {
    fn create_depth_texture<T: DepthTexture>(&self) -> T {
        T::create(self)
    }
}

pub struct DefaultDepthTexture {
    texture: Texture,
}
impl DepthTexture for DefaultDepthTexture {
    fn create(renderer: &Renderer) -> Self {
        let texture = Self::create_texture(renderer);
        Self { texture }
    }
    fn stencil_attachment(&self) -> wgpu::RenderPassDepthStencilAttachment {
        let texture = &self.texture;
        wgpu::RenderPassDepthStencilAttachment {
            view: texture
                .texture_view
                .as_ref()
                .expect("Texture view in depth texture not initilized"),
            depth_ops: Some(wgpu::Operations {
                load: wgpu::LoadOp::Clear(1.0),
                store: wgpu::StoreOp::Store,
            }),
            stencil_ops: None,
        }
    }
}

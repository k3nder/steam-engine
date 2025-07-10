use super::Identifier;
use super::ResourceLoader;
use hashbrown::HashMap;
use steamengine_renderer::Renderer;
use steamengine_renderer::texture::TextureBuilder;
use steamengine_renderer::texture::TextureDimensions;
use tracing::*;
use wgpu::TextureFormat;
use wgpu::TextureUsages;

/// Bound of a texture inside the atlas
#[derive(Copy, Clone)]
pub struct TextureBounds {
    pub uv_offset: [f32; 2],
    pub uv_scale: [f32; 2],
}

/// Implementation of Resource loader for textures
pub struct TextureResourceLoader;
impl TextureResourceLoader {
    pub fn new() -> Self {
        Self
    }
    /// load all the textures inside a atlas and gets the bounds
    pub fn load_to_atlas(
        &self,
        root: &str,
        renderer: std::sync::Arc<Renderer>,
    ) -> (
        crate::bindings::Bindings,
        HashMap<Identifier, TextureBounds>,
    ) {
        let images: HashMap<Identifier, image::DynamicImage> =
            self.load_all(root).expect("Cannot read textures");

        let mut atlas_dim_width = 0;
        let mut atlas_dim_height = 0;

        for (_, image) in &images {
            let dim = image.to_rgba8();
            let (width, height) = dim.dimensions();

            atlas_dim_width += width + 10;

            if height > atlas_dim_height {
                atlas_dim_height = height;
            }
        }

        info!("ATLAS WIDTH : {}", atlas_dim_width);
        info!("ATLAS HEIGHT : {}", atlas_dim_height);

        let mut x = 0;
        let mut bounds: HashMap<Identifier, TextureBounds> = HashMap::new();
        let mut atlas = vec![0u8; atlas_dim_width as usize * atlas_dim_height as usize * 4];

        for (id, image) in images {
            let img = image.to_rgba8();
            let (width, height) = img.dimensions();
            let pixels = img.as_raw();

            let pos_x_raw = x;

            let pos_x = pos_x_raw as f32 / atlas_dim_width as f32;
            let pos_y = 0.0;

            let scale_x = width as f32 / atlas_dim_width as f32;
            let scale_y = height as f32 / atlas_dim_height as f32;

            bounds.insert(
                id,
                TextureBounds {
                    uv_offset: [pos_x, pos_y],
                    uv_scale: [scale_x, scale_y],
                },
            );

            for y in 0..height {
                let dst_start = ((y * atlas_dim_width + pos_x_raw) * 4) as usize;
                let src_start = (y * width * 4) as usize;

                atlas[dst_start..dst_start + (width * 4) as usize]
                    .copy_from_slice(&pixels[src_start..src_start + (width * 4) as usize]);
            }

            x += width + 10;
        }

        let mut texture = renderer.init_texture(
            "Global Texture Atlas",
            None,
            TextureBuilder::new()
                .data(atlas)
                .usage(TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST)
                .format(TextureFormat::Rgba8UnormSrgb)
                .dimension(TextureDimensions::new_2d(atlas_dim_width, atlas_dim_height))
                .mip_level(0)
                .origin(wgpu::Origin3d::ZERO)
                .aspect(wgpu::TextureAspect::All)
                .buffer_offset(0)
                .bytes_per_row((atlas_dim_width * 4) as u32)
                .rows_per_image(atlas_dim_height),
        );
        texture.texture_view(wgpu::TextureViewDescriptor::default());
        texture.texture_sampler(
            wgpu::SamplerDescriptor {
                address_mode_u: wgpu::AddressMode::ClampToEdge,
                address_mode_v: wgpu::AddressMode::ClampToEdge,
                address_mode_w: wgpu::AddressMode::ClampToEdge,
                mag_filter: wgpu::FilterMode::Linear,
                min_filter: wgpu::FilterMode::Nearest,
                mipmap_filter: wgpu::FilterMode::Nearest,
                ..Default::default()
            },
            &renderer,
        );

        let (layout, bind) =
            texture.default_bind_group("Global Texture Atlas Bind Group", &renderer);

        let atlas = crate::bindings::Bindings::new(bind, layout);

        (atlas, bounds)
    }
}

impl ResourceLoader for TextureResourceLoader {
    type Resource = image::DynamicImage;
    type Error = crate::errors::Error;
    fn label(&self) -> &'static str {
        "Texture Resource Loader"
    }
    fn load_from_bytes(&self, bytes: Vec<u8>) -> Result<Self::Resource, Self::Error> {
        let image = image::load_from_memory(&bytes)?;
        Ok(image)
    }
}

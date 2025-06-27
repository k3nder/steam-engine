/// This module constrains an api to communicate to WGPU
use std::{fs::File, io::Read};

use bind_group::BindGroupEntryBuilder;
use bytemuck::NoUninit;
use errors::{RendererSetupError, TextureError};
use texture::{Texture, TextureBuilder, TextureDimensions};
use tracing::*;
use vertex::Vertex;
use wgpu::{
    BackendOptions, Backends, BindGroupEntry, BindGroupLayoutDescriptor, BindGroupLayoutEntry,
    Buffer, BufferDescriptor, BufferUsages, CommandEncoder, InstanceFlags, PresentMode,
    SurfaceTexture, TextureFormat, TextureView, TextureViewDescriptor, Trace,
    util::{BufferInitDescriptor, DeviceExt},
};

use winit::window::Window;

/// This module is an utility to build bind groups
pub mod bind_group;
/// This module contrains the errors
pub mod errors;
/// This module contrains a macro to build a simple render pass
#[macro_use]
pub mod render_pass;
pub mod instances;
/// This module contrains an utilities to create a render pipeline
pub mod render_pipeline;
/// This module contrains a utility to create textures
pub mod texture;
/// This module contrains an utilities to load vertex
pub mod vertex;

/// This the builder of the renderer, in this builder you can to set the parameters of the renderer Ex: performance mode
pub struct RendererBuilder {
    backends: Backends,
    flags: InstanceFlags,
    backend_options: BackendOptions,
    power_preference: wgpu::PowerPreference,
    force_fallback_adapter: bool,
    required_features: wgpu::Features,
    required_limits: wgpu::Limits,
    memory_hints: wgpu::MemoryHints,
    surface_format: fn(caps: &wgpu::SurfaceCapabilities) -> wgpu::TextureFormat,
    usage: wgpu::TextureUsages,
    present_mode: fn(caps: &wgpu::SurfaceCapabilities) -> PresentMode,
    alpha_mode: fn(caps: &wgpu::SurfaceCapabilities) -> wgpu::CompositeAlphaMode,
    view_formats: Vec<wgpu::TextureFormat>,
    trace: Trace,
    desired_maximum_frame_latency: u32,
}
impl RendererBuilder {
    pub fn new() -> Self {
        RendererBuilder {
            backends: Backends::all(),
            flags: InstanceFlags::empty(),
            backend_options: BackendOptions::default(),
            power_preference: wgpu::PowerPreference::default(),
            force_fallback_adapter: false,
            required_features: wgpu::Features::empty(),
            required_limits: wgpu::Limits::downlevel_defaults(),
            memory_hints: wgpu::MemoryHints::default(),
            surface_format: |caps| {
                caps.formats
                    .iter()
                    .copied()
                    .find(|f| f.is_srgb())
                    .unwrap_or(caps.formats[0])
            },
            trace: Trace::Off,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            present_mode: |caps| caps.present_modes[0],
            alpha_mode: |caps| caps.alpha_modes[0],
            view_formats: Vec::new(),
            desired_maximum_frame_latency: 2,
        }
    }
    /// Sets the backend of wgpu, example, Vulkan or OpenGL
    pub fn backends(mut self, backends: Backends) -> Self {
        self.backends = backends;
        self
    }
    /// Sets the flags of the renderer
    pub fn flags(mut self, flags: InstanceFlags) -> Self {
        self.flags = flags;
        self
    }
    /// Sets the options of the backend
    pub fn backend_options(mut self, backend_options: BackendOptions) -> Self {
        self.backend_options = backend_options;
        self
    }
    /// Sets the performance mode
    pub fn power_preference(mut self, power_preference: wgpu::PowerPreference) -> Self {
        self.power_preference = power_preference;
        self
    }
    pub fn force_fallback_adapter(mut self, force_fallback_adapter: bool) -> Self {
        self.force_fallback_adapter = force_fallback_adapter;
        self
    }
    pub fn required_features(mut self, required_features: wgpu::Features) -> Self {
        self.required_features = required_features;
        self
    }
    pub fn required_limits(mut self, required_limits: wgpu::Limits) -> Self {
        self.required_limits = required_limits;
        self
    }
    pub fn memory_hints(mut self, memory_hints: wgpu::MemoryHints) -> Self {
        self.memory_hints = memory_hints;
        self
    }
    pub fn surface_format(
        mut self,
        surface_format: fn(caps: &wgpu::SurfaceCapabilities) -> wgpu::TextureFormat,
    ) -> Self {
        self.surface_format = surface_format;
        self
    }
    pub fn usage(mut self, usage: wgpu::TextureUsages) -> Self {
        self.usage = usage;
        self
    }
    pub fn present_mode(
        mut self,
        present_mode: fn(caps: &wgpu::SurfaceCapabilities) -> PresentMode,
    ) -> Self {
        self.present_mode = present_mode;
        self
    }
    pub fn alpha_mode(
        mut self,
        alpha_mode: fn(caps: &wgpu::SurfaceCapabilities) -> wgpu::CompositeAlphaMode,
    ) -> Self {
        self.alpha_mode = alpha_mode;
        self
    }
    pub fn view_formats(mut self, view_formats: Vec<wgpu::TextureFormat>) -> Self {
        self.view_formats = view_formats;
        self
    }
    pub fn desired_maximum_frame_latency(mut self, desired_maximum_frame_latency: u32) -> Self {
        self.desired_maximum_frame_latency = desired_maximum_frame_latency;
        self
    }
    pub fn trace(mut self, trace: Trace) -> Self {
        self.trace = trace;
        self
    }
    pub async fn build<'a>(
        self,
        window: std::sync::Arc<Window>,
        size: (u32, u32),
    ) -> Result<Renderer<'a>, RendererSetupError> {
        // The instance is a handle to our GPU
        // BackendBit::PRIMARY => Vulkan + Metal + DX12 + Browser WebGPU
        trace!("Creating renderer");
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: self.backends,
            flags: self.flags,
            backend_options: self.backend_options,
        });

        let surface = instance.create_surface(window)?;

        trace!("Surface created");
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: self.power_preference,
                compatible_surface: Some(&surface),
                force_fallback_adapter: self.force_fallback_adapter,
            })
            .await?;
        let features = adapter.features();
        if !features.contains(self.required_features) {
            error!("The device dont support the features")
        }
        trace!("Adapter created");
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: None,
                required_features: self.required_features,
                // WebGL doesn't support all of wgpu's features, so if
                // we're building for the web we'll have to disable some.
                required_limits: self.required_limits,
                memory_hints: self.memory_hints,
                trace: self.trace,
            })
            .await?;
        trace!("Device and Queue created");

        let surface_caps = surface.get_capabilities(&adapter);
        // Shader code in this tutorial assumes an Srgb surface texture. Using a different
        // one will result all the colors coming out darker. If you want to support non
        // Srgb surfaces, you'll need to account for that when drawing to the frame.
        let surface_format = (self.surface_format)(&surface_caps);
        let config = wgpu::SurfaceConfiguration {
            usage: self.usage,
            format: surface_format,
            width: size.0,
            height: size.1,
            present_mode: (self.present_mode)(&surface_caps),
            alpha_mode: (self.alpha_mode)(&surface_caps),
            view_formats: self.view_formats,
            desired_maximum_frame_latency: self.desired_maximum_frame_latency,
        };
        trace!("Config created");
        let surface = std::sync::RwLock::new(surface);
        let config = std::sync::RwLock::new(config);
        let size = std::sync::RwLock::new(size);
        trace!("Renderer builded");
        Ok(Renderer {
            surface,
            device,
            queue,
            config,
            size,
        })
    }
}
/// this struct contrais all the components to render
pub struct Renderer<'a> {
    pub surface: std::sync::RwLock<wgpu::Surface<'a>>,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub config: std::sync::RwLock<wgpu::SurfaceConfiguration>,
    pub size: std::sync::RwLock<(u32, u32)>,
}

impl<'a> Renderer<'a> {
    /// create a new render_pass encoder
    pub fn create_encoder(
        &self,
    ) -> Result<(CommandEncoder, TextureView, SurfaceTexture), wgpu::SurfaceError> {
        trace!("Renderer creating encoder");
        let output = self
            .surface
            .read()
            .expect("Cannot read surface")
            .get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        Ok((
            self.device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Render Encoder"),
                }),
            view,
            output,
        ))
    }
    /// gets the surface
    pub fn surface(&self) -> std::sync::RwLockReadGuard<wgpu::Surface> {
        self.surface.read().expect("Cannot read surface")
    }
    /// gets the device
    pub fn device(&self) -> &wgpu::Device {
        &self.device
    }
    /// gets the queue
    pub fn queue(&self) -> &wgpu::Queue {
        &self.queue
    }
    /// gets the config
    pub fn config(&self) -> std::sync::RwLockReadGuard<wgpu::SurfaceConfiguration> {
        self.config.read().expect("Cannot read config")
    }
    /// gets the size
    pub fn size(&self) -> (u32, u32) {
        self.size.read().unwrap().clone()
    }
    pub fn resize(&self, new_size: &(u32, u32)) {
        if new_size.0 > 0 && new_size.1 > 0 {
            *self.size.write().expect("Cannot write size") = new_size.clone();
            self.config.write().expect("Cannot write config").width = new_size.0;
            self.config.write().expect("Cannot write config").height = new_size.1;
            self.surface
                .write()
                .unwrap()
                .configure(&self.device, &self.config.read().unwrap());
        }
    }
    /// init a new buffer with a data
    pub fn init_buffer<A>(&self, label: &str, usage: BufferUsages, content: &[A]) -> Buffer
    where
        A: NoUninit,
    {
        self.device.create_buffer_init(&BufferInitDescriptor {
            label: Some(label),
            contents: bytemuck::cast_slice(content),
            usage,
        })
    }
    pub fn create_buffer(&self, label: &str, usage: BufferUsages, size: u64) -> Buffer {
        self.device().create_buffer(&BufferDescriptor {
            size,
            usage,
            label: Some(label),
            mapped_at_creation: false,
        })
    }
    /// create a new bind group
    pub fn bind_group(
        &self,
        label: &str,
        entries: &[BindGroupEntryBuilder],
    ) -> (wgpu::BindGroup, wgpu::BindGroupLayout) {
        trace!("Renderer building bind group -- {}", label);
        let layout_entries: Vec<wgpu::BindGroupLayoutEntry> = entries
            .iter()
            .map(|entry| BindGroupLayoutEntry {
                binding: entry.binding,
                visibility: entry.visibility,
                ty: entry.ty,
                count: entry.count,
            })
            .collect();
        trace!("With {} entries -- {}", layout_entries.len(), label);
        let layout = self
            .device
            .create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some(label),
                entries: &layout_entries,
            });
        trace!("Layout created -- {}", label);
        let entries: Vec<_> = entries
            .iter()
            .map(|entry| BindGroupEntry {
                binding: entry.binding,
                resource: entry
                    .resource
                    .clone()
                    .expect("Resource binding in bind group not defined"),
            })
            .collect();
        trace!("{} Entries builded into BindGroupEntry", label);
        let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some(label),
            layout: &layout,
            entries: &entries,
        });
        trace!("Finnish bind group creation -- {}", label);
        (bind_group, layout)
    }
    /// init a new texture
    pub fn init_texture(
        &self,
        label: &'static str,
        view_formats: Option<&'static [TextureFormat]>,
        builder: TextureBuilder,
    ) -> Texture {
        builder.build(label, view_formats, self)
    }
    /// simple load a png texture from file
    pub fn simple_png_texture_file(&self, file: &mut File) -> Result<Texture, TextureError> {
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;
        self.simple_png_texture_bytes(buffer.as_slice())
    }
    /// simple load a png texture from bytes
    pub fn simple_png_texture_bytes(&self, bytes: &[u8]) -> Result<Texture, TextureError> {
        let diffuse_image = image::load_from_memory(bytes)?;
        let diffuse_rgba = diffuse_image.to_rgba8();
        use image::GenericImageView;
        let dimensions = diffuse_image.dimensions();

        let mut texture = self.init_texture(
            "texture",
            None,
            TextureBuilder::new()
                .dimension(TextureDimensions::D2(dimensions.0, dimensions.1))
                .data(diffuse_rgba.to_vec()),
        );

        texture.texture_view(TextureViewDescriptor::default());

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
            self,
        );

        texture.default_bind_group("texture bind group", self);
        Ok(texture)
    }
    /// Load vertices to buffer
    pub fn init_buffer_from_vertices<A: Vertex>(&self, label: &str, vertices: &[A]) -> Buffer {
        self.init_buffer(label, BufferUsages::VERTEX, vertices)
    }
    /// Load indices to buffer
    pub fn init_buffer_from_indices<A: NoUninit>(&self, label: &str, indices: &[A]) -> Buffer {
        self.init_buffer(label, BufferUsages::INDEX, indices)
    }
    /// Load a index buffer and vertex buffer
    /// Return: (vertex buffer, index buffer)
    pub fn init_buffers_from_model<A: Vertex, B: NoUninit>(
        &self,
        label: &str,
        vertices: &[A],
        indices: &[B],
    ) -> (Buffer, Buffer) {
        trace!(
            "Initializing model buffers with {} vertices and {} indices -- {}",
            vertices.len(),
            indices.len(),
            label
        );
        let vertices_label = format!("{} - VERTICES", label);
        let indices_label = format!("{} - INDICES", label);

        let vertices_buffer = self.init_buffer_from_vertices(vertices_label.as_str(), vertices);
        let indices_buffer = self.init_buffer_from_indices(indices_label.as_str(), indices);

        (vertices_buffer, indices_buffer)
    }
    pub fn update_buffer<A: NoUninit>(&self, buffer: &Buffer, data: &[A]) {
        self.queue()
            .write_buffer(buffer, 0, bytemuck::cast_slice(data));
    }
    pub fn update_buffer_entry<A: NoUninit>(&self, buffer: &Buffer, id: u64, data: A) {
        let size = std::mem::size_of::<A>();
        let offset = size as u64 * id;
        self.queue()
            .write_buffer(buffer, offset, bytemuck::cast_slice(&[data]));
    }
}

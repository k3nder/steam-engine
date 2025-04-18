use wgpu::{
    BackendOptions, Backends, CommandEncoder, InstanceFlags, PresentMode, SurfaceTexture,
    TextureView, Trace,
};
use winit::dpi::PhysicalSize;

pub mod render_pass;

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
    pub fn backends(mut self, backends: Backends) -> Self {
        self.backends = backends;
        self
    }
    pub fn flags(mut self, flags: InstanceFlags) -> Self {
        self.flags = flags;
        self
    }
    pub fn backend_options(mut self, backend_options: BackendOptions) -> Self {
        self.backend_options = backend_options;
        self
    }
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
    pub async fn build<'a>(self, window: &'a winit::window::Window) -> Renderer<'a> {
        let size = window.inner_size();

        // The instance is a handle to our GPU
        // BackendBit::PRIMARY => Vulkan + Metal + DX12 + Browser WebGPU
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: self.backends,
            flags: self.flags,
            backend_options: self.backend_options,
        });

        let surface = instance.create_surface(window).unwrap();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: self.power_preference,
                compatible_surface: Some(&surface),
                force_fallback_adapter: self.force_fallback_adapter,
            })
            .await
            .unwrap();

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
            .await
            .unwrap();

        let surface_caps = surface.get_capabilities(&adapter);
        // Shader code in this tutorial assumes an Srgb surface texture. Using a different
        // one will result all the colors comming out darker. If you want to support non
        // Srgb surfaces, you'll need to account for that when drawing to the frame.
        let surface_format = (self.surface_format)(&surface_caps);
        let config = wgpu::SurfaceConfiguration {
            usage: self.usage,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: (self.present_mode)(&surface_caps),
            alpha_mode: (self.alpha_mode)(&surface_caps),
            view_formats: self.view_formats,
            desired_maximum_frame_latency: self.desired_maximum_frame_latency,
        };

        Renderer {
            surface,
            device,
            queue,
            config,
            size,
            window,
        }
    }
}

pub struct Renderer<'a> {
    pub surface: wgpu::Surface<'a>,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
    pub size: winit::dpi::PhysicalSize<u32>,
    pub window: &'a winit::window::Window,
}

impl<'a> Renderer<'a> {
    pub fn create_encoder(
        &self,
    ) -> Result<(CommandEncoder, TextureView, SurfaceTexture), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
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
    pub fn surface(&self) -> &wgpu::Surface {
        &self.surface
    }
    pub fn device(&self) -> &wgpu::Device {
        &self.device
    }
    pub fn queue(&self) -> &wgpu::Queue {
        &self.queue
    }
    pub fn config(&self) -> &wgpu::SurfaceConfiguration {
        &self.config
    }
    pub fn size(&self) -> PhysicalSize<u32> {
        self.size.clone()
    }
    pub fn window(&self) -> &winit::window::Window {
        &self.window
    }
    pub fn resize(&mut self, new_size: &PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size.clone();
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }
}

use std::iter;

use rand::Rng;
use steamengine::{
    color_render_pass, exec,
    render::{
        Renderer, RendererBuilder,
        render_pipeline::RenderPipeline,
        texture::{Texture, TextureBuilder, TextureDimensions},
        vertex::Vertex,
    },
    thread,
    threads::channel::{CommManager, Event, Message},
    windows::{
        AppHandle,
        errors::{CalculationError, RenderError, SetupError},
    },
};
use wgpu::{
    BindGroupLayout, BufferUsages, TextureDimension, TextureViewDescriptor, VertexBufferLayout,
};
use winit::{event_loop::EventLoopWindowTarget, window::WindowBuilder};

const VERTICES: &[Vertex3DTexture] = &[
    Vertex3DTexture {
        position: [-0.0868241, 0.49240386, 0.0],
        tex_coords: [0.4131759, 0.99240386],
    }, // A
    Vertex3DTexture {
        position: [-0.49513406, 0.06958647, 0.0],
        tex_coords: [0.0048659444, 0.56958647],
    }, // B
    Vertex3DTexture {
        position: [-0.21918549, -0.44939706, 0.0],
        tex_coords: [0.28081453, 0.05060294],
    }, // C
    Vertex3DTexture {
        position: [0.35966998, -0.3473291, 0.0],
        tex_coords: [0.85967, 0.1526709],
    }, // D
    Vertex3DTexture {
        position: [0.44147372, 0.2347359, 0.0],
        tex_coords: [0.9414737, 0.7347359],
    }, // E
];
const INDICES: &[u16] = &[0, 1, 4, 1, 2, 4, 2, 3, 4, /* padding */ 0];

struct App {
    threads: CommManager,
    color: (f64, f64, f64),
    render_pipeline: Option<wgpu::RenderPipeline>,
    vertex_buffer: Option<wgpu::Buffer>,
    index_buffer: Option<wgpu::Buffer>,
    texture: Option<Texture>,
}
impl AppHandle for App {
    fn redraw(
        &mut self,
        renderer: &Renderer,
        _control: &EventLoopWindowTarget<()>,
    ) -> Result<(), RenderError> {
        let (mut encoder, view, output) = renderer.create_encoder()?;
        {
            let mut _render_pass = encoder.begin_render_pass(&color_render_pass!(1.0, view));

            let texture = self.texture.as_ref().unwrap();
            let bind_group = texture.bind_group.as_ref().unwrap();

            _render_pass.set_pipeline(
                self.render_pipeline
                    .as_ref()
                    .expect("Render Pipeline Not initialized"),
            );
            _render_pass.set_bind_group(0, bind_group, &[]);
            _render_pass.set_vertex_buffer(
                0,
                self.vertex_buffer
                    .as_ref()
                    .expect("Vertex Buffer Not initialized")
                    .slice(..),
            );

            _render_pass.draw(0..3, 0..1); // 3.
        }

        renderer.queue().submit(iter::once(encoder.finish()));
        output.present();

        Ok(())
    }

    fn update(&mut self, _control: &EventLoopWindowTarget<()>) -> Result<(), CalculationError> {
        Ok(())
    }
    fn on_resize(
        &mut self,
        new_size: &winit::dpi::PhysicalSize<u32>,
        renderer: &mut Renderer,
        _: &EventLoopWindowTarget<()>,
    ) -> bool {
        renderer.resize(new_size);
        true
    }
    fn on_close(&mut self, control: &EventLoopWindowTarget<()>) -> bool {
        self.threads.broadcast(Message::Event(Event::Exit)).unwrap();
        control.exit();
        true
    }
    fn on_keyboard(
        &mut self,
        key: winit::keyboard::Key,
        control: &EventLoopWindowTarget<()>,
    ) -> bool {
        self.threads.send_to(1, Message::Int(3)).unwrap();
        match key {
            winit::keyboard::Key::Character(_) => {
                self.color.0 = if let Ok(Message::Float(color)) = self.threads.recv() {
                    color as f64
                } else {
                    self.color.0
                };
                self.color.1 = if let Ok(Message::Float(color)) = self.threads.recv() {
                    color as f64
                } else {
                    self.color.1
                };
                self.color.2 = if let Ok(Message::Float(color)) = self.threads.recv() {
                    color as f64
                } else {
                    self.color.2
                };
                true
            }
            winit::keyboard::Key::Named(c) => {
                match c {
                    winit::keyboard::NamedKey::Escape => {
                        self.on_close(control);
                    }
                    _ => {}
                }
                true
            }
            _ => false,
        }
    }
    fn window(&self) -> winit::window::WindowBuilder {
        WindowBuilder::new().with_title("Windows")
    }

    fn setup(&mut self, renderer: &Renderer) -> Result<(), SetupError> {
        self.vertex_buffer =
            Some(renderer.init_buffer("vertex buffer", BufferUsages::VERTEX, VERTICES));
        self.index_buffer =
            Some(renderer.init_buffer("index buffer", BufferUsages::INDEX, INDICES));

        let diffuse_bytes = include_bytes!("resources/happy-tree.png");
        let diffuse_image = image::load_from_memory(diffuse_bytes).unwrap();
        let diffuse_rgba = diffuse_image.to_rgba8();
        use image::GenericImageView;
        let dimensions = diffuse_image.dimensions();

        self.texture = Some(
            renderer.init_texture(
                "texture",
                None,
                TextureBuilder::new()
                    .dimension(TextureDimensions::D2(dimensions.0, dimensions.1))
                    .data(diffuse_rgba),
            ),
        );

        self.texture
            .as_mut()
            .unwrap()
            .texture_view(TextureViewDescriptor::default());

        self.texture.as_mut().unwrap().texture_sampler(
            wgpu::SamplerDescriptor {
                address_mode_u: wgpu::AddressMode::ClampToEdge,
                address_mode_v: wgpu::AddressMode::ClampToEdge,
                address_mode_w: wgpu::AddressMode::ClampToEdge,
                mag_filter: wgpu::FilterMode::Linear,
                min_filter: wgpu::FilterMode::Nearest,
                mipmap_filter: wgpu::FilterMode::Nearest,
                ..Default::default()
            },
            renderer,
        );

        self.texture
            .as_mut()
            .unwrap()
            .default_bind_group("texture bind group", renderer);

        self.render_pipeline = Some(
            Vertex3DColorPipeline::new(&[self
                .texture
                .as_ref()
                .unwrap()
                .bind_group_layout
                .as_ref()
                .unwrap()])
            .to_wgpu(renderer),
        );

        Ok(())
    }
}

fn main() {
    env_logger::init();
    let mut comm_manager = CommManager::new();
    let random = thread!(
        1,
        comm_manager,
        |channel: steamengine::threads::channel::Channel| {
            let mut rng = rand::thread_rng();
            loop {
                let rec = channel.recv();
                if let Ok(message) = rec {
                    match message {
                        Message::Int(num) => {
                            for _ in 0..num {
                                channel.send(0, Message::Float(rng.r#gen())).unwrap();
                            }
                        }
                        Message::Event(steamengine::threads::channel::Event::Exit) => {
                            break;
                        }
                        _ => {}
                    }
                }
            }
        }
    );
    let app = App {
        threads: comm_manager,
        color: (0.0, 0.0, 0.0),
        render_pipeline: None,
        vertex_buffer: None,
        texture: None,
        index_buffer: None,
    };
    pollster::block_on(exec!(app, RendererBuilder::new()));
    random.join().unwrap();
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex3DTexture {
    pub position: [f32; 3],
    pub tex_coords: [f32; 2],
}
impl Vertex for Vertex3DTexture {
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                },
            ],
        }
    }
}

pub struct Vertex3DColorPipeline<'a> {
    bind_group_layouts: &'a [&'a BindGroupLayout],
}

impl<'a> Vertex3DColorPipeline<'a> {
    pub fn new(bind_group_layouts: &'a [&'a BindGroupLayout]) -> Self {
        Self { bind_group_layouts }
    }
}

impl<'a> RenderPipeline for Vertex3DColorPipeline<'a> {
    fn label(&self) -> &str {
        "Vertex 3D color render pipeline"
    }

    fn source(&self) -> &str {
        r#"
        // Vertex shader

        struct VertexInput {
            @location(0) position: vec3<f32>,
            @location(1) tex_coords: vec2<f32>,
        }

        struct VertexOutput {
            @builtin(position) clip_position: vec4<f32>,
            @location(0) tex_coords: vec2<f32>,
        }

        @vertex
        fn vs_main(
            model: VertexInput,
        ) -> VertexOutput {
            var out: VertexOutput;
            out.tex_coords = model.tex_coords;
            out.clip_position = vec4<f32>(model.position, 1.0);
            return out;
        }

        // Fragment shader

        @group(0) @binding(0)
        var t_diffuse: texture_2d<f32>;
        @group(0) @binding(1)
        var s_diffuse: sampler;

        @fragment
        fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
            return textureSample(t_diffuse, s_diffuse, in.tex_coords);
        }
        "#
    }
    fn buffers(&self) -> Vec<VertexBufferLayout> {
        vec![Vertex3DTexture::desc()]
    }
    fn layout(&self) -> wgpu::PipelineLayoutDescriptor {
        wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: self.bind_group_layouts,
            push_constant_ranges: &[],
        }
    }
}

use std::iter;

use rand::Rng;
use steamengine::{
    color_render_pass, exec,
    render::{Renderer, RendererBuilder, render_pipeline::RenderPipeline},
    thread,
    threads::channel::{CommManager, Event, Message},
    vertex::Vertex,
    windows::AppHandle,
};
use wgpu::{BufferUsages, VertexBufferLayout};
use winit::{event_loop::EventLoopWindowTarget, window::WindowBuilder};

const VERTICES: &[Vertex3DColor] = &[
    Vertex3DColor {
        position: [0.0, 0.5, 0.0],
        color: [1.0, 0.0, 0.0],
    },
    Vertex3DColor {
        position: [-0.5, -0.5, 0.0],
        color: [0.0, 1.0, 0.0],
    },
    Vertex3DColor {
        position: [0.5, -0.5, 0.0],
        color: [0.0, 0.0, 1.0],
    },
];

struct App {
    threads: CommManager,
    color: (f64, f64, f64),
    render_pipeline: Option<wgpu::RenderPipeline>,
    vertex_buffer: Option<wgpu::Buffer>,
}
impl AppHandle for App {
    fn redraw(
        &mut self,
        renderer: &Renderer,
        _control: &EventLoopWindowTarget<()>,
    ) -> Result<(), wgpu::SurfaceError> {
        let (mut encoder, view, output) = renderer.create_encoder().unwrap();
        {
            let mut _render_pass = encoder.begin_render_pass(&color_render_pass!(1.0, view));

            _render_pass.set_pipeline(
                self.render_pipeline
                    .as_ref()
                    .expect("Render Pipeline Not initialized"),
            );
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

    fn update(&mut self, _control: &EventLoopWindowTarget<()>) {
        self.threads.send_to(1, Message::Int(3)).unwrap();

        self.color.0 = if let Ok(Message::Float(color)) = self.threads.try_recv() {
            color as f64
        } else {
            self.color.0
        };
        self.color.1 = if let Ok(Message::Float(color)) = self.threads.try_recv() {
            color as f64
        } else {
            self.color.1
        };
        self.color.2 = if let Ok(Message::Float(color)) = self.threads.try_recv() {
            color as f64
        } else {
            self.color.2
        };
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

    fn setup(&mut self, renderer: &Renderer) {
        self.render_pipeline = Some(Vertex3DColorPipeline::new().to_wgpu(renderer));
        self.vertex_buffer =
            Some(renderer.init_buffer("vertex buffer", BufferUsages::VERTEX, VERTICES))
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
    };
    pollster::block_on(exec!(app, RendererBuilder::new()));
    random.join().unwrap();
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex3DColor {
    pub position: [f32; 3],
    pub color: [f32; 3],
}
impl Vertex for Vertex3DColor {
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
                    format: wgpu::VertexFormat::Float32x3,
                },
            ],
        }
    }
}

pub struct Vertex3DColorPipeline;

impl Vertex3DColorPipeline {
    pub fn new() -> Self {
        Self {}
    }
}

impl RenderPipeline for Vertex3DColorPipeline {
    fn label(&self) -> &str {
        "Vertex 3D color render pipeline"
    }

    fn source(&self) -> &str {
        r#"
        // Vertex shader

        struct VertexInput {
            @location(0) position: vec3<f32>,
            @location(1) color: vec3<f32>,
        };

        struct VertexOutput {
            @builtin(position) clip_position: vec4<f32>,
            @location(0) color: vec3<f32>,
        };

        @vertex
        fn vs_main(
            model: VertexInput,
        ) -> VertexOutput {
            var out: VertexOutput;
            out.color = model.color;
            out.clip_position = vec4<f32>(model.position, 1.0);
            return out;
        }

        // Fragment shader

        @fragment
        fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
            return vec4<f32>(in.color, 1.0);
        }

        "#
    }
    fn buffers(&self) -> Vec<VertexBufferLayout> {
        vec![Vertex3DColor::desc()]
    }
}

use std::{fs::File, iter};

use steamengine::render::render_pipeline::RenderPipeline;
use steamengine::{
    color_render_pass, exec,
    render::{
        Renderer, RendererBuilder, render_pipeline::basic::BasicTexturePipeline, texture::Texture,
        vertex::VertexBasicWithTexture,
    },
    windows::{
        AppHandle,
        errors::{CalculationError, RenderError, SetupError},
    },
};
use winit::{event_loop::EventLoopWindowTarget, window::WindowBuilder};

const VERTICES: &[VertexBasicWithTexture] = &[
    VertexBasicWithTexture {
        position: [-0.0868241, 0.49240386, 0.0],
        tex_coords: [0.4131759, 0.99240386],
    }, // A
    VertexBasicWithTexture {
        position: [-0.49513406, 0.06958647, 0.0],
        tex_coords: [0.0048659444, 0.56958647],
    }, // B
    VertexBasicWithTexture {
        position: [-0.21918549, -0.44939706, 0.0],
        tex_coords: [0.28081453, 0.05060294],
    }, // C
    VertexBasicWithTexture {
        position: [0.35966998, -0.3473291, 0.0],
        tex_coords: [0.85967, 0.1526709],
    }, // D
    VertexBasicWithTexture {
        position: [0.44147372, 0.2347359, 0.0],
        tex_coords: [0.9414737, 0.7347359],
    }, // E
];
const INDICES: &[u16] = &[0, 1, 4, 1, 2, 4, 2, 3, 4, /* padding */ 0];

struct App {
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
            _render_pass.set_index_buffer(
                self.index_buffer
                    .as_ref()
                    .expect("Index buffer not initialized")
                    .slice(..),
                wgpu::IndexFormat::Uint16,
            );

            _render_pass.draw_indexed(0..INDICES.len() as u32, 0, 0..1); // 3.
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
    fn window(&self) -> winit::window::WindowBuilder {
        WindowBuilder::new().with_title("Steam engine example")
    }

    fn setup(&mut self, renderer: &Renderer) -> Result<(), SetupError> {
        let (vertices, indices) = renderer.init_buffers_from_model("Triangle", VERTICES, INDICES);
        self.vertex_buffer = Some(vertices);
        self.index_buffer = Some(indices);

        self.texture = Some(renderer.simple_png_texture_file(&mut File::open(
            "/home/kristian/Escritorio/steamengine/examples/resources/happy-tree.png",
        )?)?);

        let texture = self.texture.as_ref().unwrap();
        let texture_bind_group_layout = texture.bind_group_layout.as_ref().unwrap();

        self.render_pipeline =
            Some(BasicTexturePipeline::new(&[texture_bind_group_layout]).to_wgpu(renderer));

        Ok(())
    }
}

fn main() {
    env_logger::init();
    let app = App {
        render_pipeline: None,
        vertex_buffer: None,
        texture: None,
        index_buffer: None,
    };
    pollster::block_on(exec!(app, RendererBuilder::new())).unwrap();
}

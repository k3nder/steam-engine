use std::iter;

use steamengine_core::render::render_pipeline::RenderPipeline;
use steamengine_core::wgpu::IndexFormat;
use steamengine_core::{
    color_render_pass, wgpu::Buffer, windows::AppHandle, winit::event_loop::EventLoopWindowTarget,
};

use crate::vertex::{INDICES, VERTICES};

pub struct TriangleApp {
    pub model: Option<(Buffer, Buffer)>,
    pub pipeline: Option<steamengine_core::wgpu::RenderPipeline>,
}

impl TriangleApp {
    pub fn new() -> Self {
        TriangleApp {
            model: None,
            pipeline: None,
        }
    }
}

impl AppHandle for TriangleApp {
    fn setup(
        &mut self,
        renderer: &steamengine_core::render::Renderer,
    ) -> Result<(), steamengine_core::windows::errors::SetupError> {
        self.model = Some(renderer.init_buffers_from_model("Triangle", &VERTICES, &INDICES));
        self.pipeline = Some(crate::pipeline::RenderPipeline::new().to_wgpu(renderer));
        Ok(())
    }

    fn redraw(
        &mut self,
        renderer: &steamengine_core::render::Renderer,
        _control: &EventLoopWindowTarget<()>,
    ) -> Result<(), steamengine_core::windows::errors::RenderError> {
        let (mut encoder, view, output) = renderer.create_encoder()?;
        {
            let mut render_pass =
                encoder.begin_render_pass(&color_render_pass!(0.0, 1.0, 0.0, view));

            let pipeline = self
                .pipeline
                .as_ref()
                .expect("Render pipeline not initialized");
            let (vertices, indices) = self.model.as_ref().expect("Model not initialized");

            render_pass.set_pipeline(pipeline);
            render_pass.set_vertex_buffer(0, vertices.slice(..));
            render_pass.set_index_buffer(indices.slice(..), IndexFormat::Uint16);

            render_pass.draw_indexed(0..INDICES.len() as u32, 0, 0..1);
        }

        renderer.queue().submit(iter::once(encoder.finish()));
        output.present();

        Ok(())
    }

    fn update(
        &mut self,
        _control: &EventLoopWindowTarget<()>,
    ) -> Result<(), steamengine_core::windows::errors::CalculationError> {
        Ok(())
    }
}

use std::f32::NAN;
use std::iter;

use steamengine_core::render::instances::Instance;
use steamengine_core::render::render_pipeline::RenderPipeline;
use steamengine_core::wgpu::{BufferUsages, IndexFormat};
use steamengine_core::{
    color_render_pass, wgpu::Buffer, windows::AppHandle, winit::event_loop::EventLoopWindowTarget,
};

use crate::vertex::{INDICES, VERTICES};

pub struct TriangleApp {
    pub model: Option<(Buffer, Buffer)>,
    pub pipeline: Option<steamengine_core::wgpu::RenderPipeline>,
    pub instances: Vec<crate::instances::Instance>,
    pub instance_buffer: Option<Buffer>,
}

impl TriangleApp {
    pub fn new(instances: Vec<crate::instances::Instance>) -> Self {
        TriangleApp {
            model: None,
            pipeline: None,
            instance_buffer: None,
            instances,
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

        let instances_data: Vec<crate::instances::InstanceRaw> =
            self.instances.iter().map(|v| v.to_raw()).collect();
        let instance_buffer =
            renderer.init_buffer("instances buffer", BufferUsages::VERTEX, &instances_data);

        self.instance_buffer = Some(instance_buffer);

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
                .expect("Render pipeline not initalized");
            let (vertices, indices) = self.model.as_ref().expect("Model not initialized");

            render_pass.set_pipeline(pipeline);
            render_pass.set_vertex_buffer(0, vertices.slice(..));
            render_pass.set_vertex_buffer(1, self.instance_buffer.as_ref().unwrap().slice(..));
            render_pass.set_index_buffer(indices.slice(..), IndexFormat::Uint16);

            render_pass.draw_indexed(0..INDICES.len() as u32, 0, 0..self.instances.len() as _);
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

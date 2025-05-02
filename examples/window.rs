use steamengine::{
    exec,
    render::{Renderer, RendererBuilder},
    windows::{
        AppHandle,
        errors::{CalculationError, RenderError, SetupError},
    },
};
use wgpu::{SurfaceError, wgc::command::DrawError};
use winit::event::WindowEvent;

struct App;
impl AppHandle for App {
    fn redraw(
        &mut self,
        _renderer: &Renderer,
        _control: &winit::event_loop::EventLoopWindowTarget<()>,
    ) -> Result<(), RenderError> {
        Ok(())
    }

    fn update(
        &mut self,
        _control: &winit::event_loop::EventLoopWindowTarget<()>,
    ) -> Result<(), CalculationError> {
        Ok(())
    }

    fn on(
        &mut self,
        _: &WindowEvent,
        _control: &winit::event_loop::EventLoopWindowTarget<()>,
    ) -> bool {
        false
    }
    fn on_close(&mut self, _control: &winit::event_loop::EventLoopWindowTarget<()>) -> bool {
        println!("Closing.....");
        true
    }

    fn setup(&mut self, _renderer: &Renderer) -> Result<(), SetupError> {
        Ok(())
    }
}
impl App {
    fn new() -> Self {
        Self
    }
}

fn main() {
    pollster::block_on(exec!(App::new(), RendererBuilder::new())).unwrap();
}

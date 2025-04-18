use steamengine::{
    exec,
    render::{Renderer, RendererBuilder},
    windows::AppHandle,
};
use wgpu::SurfaceError;
use winit::event::WindowEvent;

struct App;
impl AppHandle for App {
    fn redraw(
        &mut self,
        _renderer: &Renderer,
        _control: &winit::event_loop::EventLoopWindowTarget<()>,
    ) -> Result<(), SurfaceError> {
        Ok(())
    }

    fn update(&mut self, _control: &winit::event_loop::EventLoopWindowTarget<()>) {}

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

    fn setup(&mut self, _renderer: &Renderer) {}
}
impl App {
    fn new() -> Self {
        Self
    }
}

fn main() {
    pollster::block_on(exec!(App::new(), RendererBuilder::new()));
}

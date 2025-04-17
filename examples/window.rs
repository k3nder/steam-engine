use steamengine::{exec, windows::AppHandle};
use wgpu::SurfaceError;
use winit::event::WindowEvent;

struct App;
impl AppHandle for App {
    fn redraw(&mut self) -> Result<(), SurfaceError> {
        Ok(())
    }

    fn update(&mut self) {}

    fn on(&mut self, _: &WindowEvent) -> bool {
        false
    }
    fn on_close(&mut self) -> bool {
        println!("Closing.....");
        true
    }
}
impl App {
    fn new() -> Self {
        Self
    }
}

fn main() {
    exec!(App::new());
}

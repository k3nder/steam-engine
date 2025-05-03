use app::TriangleApp;
use steamengine_core::{exec, render::RendererBuilder};

mod app;
mod pipeline;
mod vertex;

fn main() {
    env_logger::init();
    pollster::block_on(async {
        let app = TriangleApp::new();
        let renderer_builder = RendererBuilder::new();
        exec!(app, renderer_builder).await.unwrap()
    });
}

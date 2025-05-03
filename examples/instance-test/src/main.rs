use std::ops::Range;

use app::TriangleApp;
use cgmath::{Quaternion, Rotation3, Vector3};
use instances::Instance;
use steamengine_core::{exec, render::RendererBuilder};

mod app;
mod instances;
mod pipeline;
mod vertex;

fn main() {
    env_logger::init();
    pollster::block_on(async {
        let app = TriangleApp::new(generate_instances(0..10));
        let renderer_builder = RendererBuilder::new();
        exec!(app, renderer_builder).await.unwrap()
    });
}
fn generate_instances(range: Range<u32>) -> Vec<Instance> {
    range
        .map(|v| {
            log::debug!("X: {}", v as f32 / 10 as f32);

            let x: f32 = 0 as f32;
            let y: f32 = v as f32 / 10 as f32;

            let position = Vector3::new(x, -y, 0 as f32);
            let rotation =
                Quaternion::from_axis_angle(cgmath::Vector3::unit_x(), cgmath::Deg(45.0));
            Instance { position, rotation }
        })
        .collect()
}

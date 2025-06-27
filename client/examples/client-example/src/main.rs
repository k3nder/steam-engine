use crate::app::State;
use tracing::*;
use tracing_subscriber::filter::LevelFilter;
use tracing_subscriber::fmt;
use tracing_subscriber::prelude::*;
use winit::event_loop::EventLoop;

mod app;
mod buffers;
mod camera_controler;
mod color;
mod pipeline;

fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(LevelFilter::INFO) // usa rust_log si está definida
        .with(fmt::layer()) // salida formateada en consola
        .init(); // inicia como el subscriber global
    debug!("Tracing initialized");

    let event_loop = EventLoop::new()?;
    debug!("Creating event loop");
    event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);
    debug!("Setting control flow to pull");
    let mut state = State::default();
    debug!("Creating state");
    debug!("Running app");
    event_loop.run_app(&mut state)?;

    Ok(())
}

# STEAM_ENGINE
lightweight graphics engine
> [!WARNING]
> This project is still in development and may contain bugs or unexpected behavior.

## Features

- **Modern Graphics Rendering**: Powered by WGPU with support for multiple graphics backends
- **Thread Management**: Built-in thread communication system for parallel processing
- **Event Handling**: Comprehensive window and input event handling
- **Flexible Rendering Pipeline**: Customizable render pipelines with WGSL shader support
- **Texture Management**: Easy texture loading and manipulation
- **Error Handling**: Robust error types for debugging and runtime stability

## Getting Started

### Dependencies

- Rust (latest stable version recommended)
- Required crates:
  - `wgpu` - WebGPU implementation
  - `winit` - Window management
  - `image` - Texture loading
  - `crossbeam-channel` - Thread communication
  - `bytemuck` - Memory manipulation
  - `thiserror` - Error handling
  - `log` - Logging

### Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
steamengine = { git = "https://github.com/yourusername/steamengine" }
```

### Basic Usage

```rust
use steamengine::{
    exec,
    render::{RendererBuilder, render_pipeline::basic::BasicRenderPipeline},
    windows::AppHandle,
};

struct MyApp {
    pipeline: Option<wgpu::RenderPipeline>,
}

impl AppHandle for MyApp {
    fn setup(&mut self, renderer: &steamengine::render::Renderer) -> Result<(), steamengine::windows::errors::SetupError> {
        self.pipeline = Some(BasicRenderPipeline::new().to_wgpu(renderer));
        Ok(())
    }

    fn redraw(
        &mut self,
        renderer: &steamengine::render::Renderer,
        _control: &winit::event_loop::EventLoopWindowTarget<()>,
    ) -> Result<(), steamengine::windows::errors::RenderError> {
        let (mut encoder, view, output) = renderer.create_encoder()?;

        // Create a render pass that clears to blue
        let mut render_pass = encoder.begin_render_pass(&color_render_pass!(0.1, 0.2, 0.3, view));

        // Draw using our pipeline
        render_pass.set_pipeline(self.pipeline.as_ref().unwrap());
        render_pass.draw(0..3, 0..1);

        drop(render_pass);
        renderer.queue().submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }

    fn window(&self) -> winit::window::WindowBuilder {
        winit::window::WindowBuilder::new()
            .with_title("My SteamEngine App")
            .with_inner_size(winit::dpi::LogicalSize::new(800, 600))
    }
}

#[tokio::main]
async fn main() -> Result<(), steamengine::windows::errors::AppError> {
    let app = MyApp { pipeline: None };
    let renderer_config = RendererBuilder::new();

    exec!(app, renderer_config).await
}
```

## Core Components

### Renderer

The renderer is the central component for graphics operations:

```rust
let renderer_builder = RendererBuilder::new()
    .power_preference(wgpu::PowerPreference::HighPerformance)
    .required_features(wgpu::Features::empty())
    .required_limits(wgpu::Limits::default());

let renderer = renderer_builder.build(&window).await?;
```

### Render Pipelines

Create custom render pipelines by implementing the `RenderPipeline` trait:

```rust
struct MyPipeline;

impl RenderPipeline for MyPipeline {
    fn label(&self) -> &str {
        "My Custom Pipeline"
    }

    fn source(&self) -> &str {
        r#"
        // Your WGSL shader code here
        @vertex
        fn vs_main(...) -> VertexOutput {
            // Vertex shader code
        }

        @fragment
        fn fs_main(...) -> @location(0) vec4<f32> {
            // Fragment shader code
        }
        "#
    }
}
```

### Thread Communication

Use the thread communication system for parallel tasks:

```rust
let comm_manager = CommManager::new();

let worker_thread = thread!(1, comm_manager, |comm| {
    loop {
        match comm.recv() {
            Ok(Message::Event(Event::Exit)) => break,
            Ok(message) => println!("Worker received: {:?}", message),
            Err(_) => break,
        }
    }
});

// Send a message to the worker thread
comm_manager.send_to(1, Message::String("Hello from main thread".to_string()));
```

## License

[MIT License](LICENSE)

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

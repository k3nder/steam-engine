use winit::{
    event::{ElementState, KeyEvent, MouseButton, WindowEvent},
    event_loop::EventLoopWindowTarget,
    keyboard::Key,
};

use crate::render::Renderer;

/// Errors module
pub mod errors {
    use thiserror::Error;

    use crate::render::errors::{RendererSetupError, TextureError};
    #[derive(Error, Debug)]
    pub enum AppError {
        #[error("Unknown error")]
        Unknown,
        #[error("Setup error")]
        Setup(#[from] SetupError),
        #[error("Render error")]
        Render(#[from] RenderError),
        #[error("Calculation error")]
        Calculation(#[from] CalculationError),
        #[error("Event loop error: {0}")]
        EventLoop(#[from] winit::error::EventLoopError),
        #[error("Window creation error")]
        WindowCreation(#[from] winit::error::OsError),
        #[error("Error setting up renderer")]
        RendererSetupError(#[from] RendererSetupError),
        #[error("IOError")]
        IO(#[from] std::io::Error),
    }

    #[derive(Error, Debug)]
    pub enum SetupError {
        #[error("Surface creation error")]
        SurfaceCreation(#[from] wgpu::SurfaceError),
        #[error("Device creation error")]
        DeviceCreation(#[from] wgpu::Error),
        #[error("IOError")]
        IO(#[from] std::io::Error),
        #[error("Texture setup error")]
        Texture(#[from] TextureError),
    }

    #[derive(Error, Debug)]
    pub enum RenderError {
        #[error("Surface error: {0}")]
        Unknown(#[from] wgpu::SurfaceError),
    }

    #[derive(Error, Debug)]
    pub enum CalculationError {
        #[error("Calculation error")]
        Calculation(),
    }
}

pub trait AppHandle {
    /// Setup and load all the items here
    fn setup(&mut self, renderer: &Renderer) -> Result<(), errors::SetupError>;
    /// draw the window here
    fn redraw(
        &mut self,
        renderer: &Renderer,
        _control: &EventLoopWindowTarget<()>,
    ) -> Result<(), errors::RenderError>;
    /// update the window here
    fn update(
        &mut self,
        _control: &EventLoopWindowTarget<()>,
        renderer: &Renderer,
    ) -> Result<(), errors::CalculationError>;
    /// this method is called when the window is closing
    fn on_close(&mut self, _control: &EventLoopWindowTarget<()>) -> bool {
        false
    }
    /// this method is called when the window is resized
    fn on_resize(
        &mut self,
        _size: &winit::dpi::PhysicalSize<u32>,
        _renderer: &mut Renderer,
        _control: &EventLoopWindowTarget<()>,
    ) -> bool {
        _renderer.resize(_size);
        false
    }
    /// this method is called when the mouse in moved
    fn on_mouse_move(&mut self, _x: i32, _y: i32, _control: &EventLoopWindowTarget<()>) -> bool {
        false
    }
    fn on_mouse_down(
        &mut self,
        _button: MouseButton,
        _x: i32,
        _y: i32,
        _control: &EventLoopWindowTarget<()>,
    ) -> bool {
        false
    }
    fn on_mouse_up(
        &mut self,
        _button: MouseButton,
        _x: i32,
        _y: i32,
        _control: &EventLoopWindowTarget<()>,
    ) -> bool {
        false
    }
    fn on_mouse_click(
        &mut self,
        _button: MouseButton,
        _x: i32,
        _y: i32,
        _control: &EventLoopWindowTarget<()>,
    ) -> bool {
        false
    }
    fn on_mouse_double_click(
        &mut self,
        _button: MouseButton,
        _x: i32,
        _y: i32,
        _control: &EventLoopWindowTarget<()>,
    ) -> bool {
        false
    }
    fn on_minimize(&mut self, _control: &EventLoopWindowTarget<()>) -> bool {
        false
    }
    fn on_maximize(&mut self, _control: &EventLoopWindowTarget<()>) -> bool {
        false
    }
    fn on_restore(&mut self, _control: &EventLoopWindowTarget<()>) -> bool {
        false
    }
    fn on_fullscreen(&mut self, _control: &EventLoopWindowTarget<()>) -> bool {
        false
    }
    fn on_exit_fullscreen(&mut self, _control: &EventLoopWindowTarget<()>) -> bool {
        false
    }
    fn on_keyboard(
        &mut self,
        _key: Key,
        _state: ElementState,
        _event: &KeyEvent,
        _control: &EventLoopWindowTarget<()>,
    ) -> bool {
        false
    }
    fn on_key_pressed(&mut self, _key: Key, _control: &EventLoopWindowTarget<()>) -> bool {
        false
    }
    fn on_key_released(&mut self, _key: Key, _control: &EventLoopWindowTarget<()>) -> bool {
        false
    }
    fn on_focus_gained(&mut self, _control: &EventLoopWindowTarget<()>) -> bool {
        false
    }
    fn on_focus_lost(&mut self, _control: &EventLoopWindowTarget<()>) -> bool {
        false
    }
    fn on_mouse_wheel(&mut self, _delta: f32, _control: &EventLoopWindowTarget<()>) -> bool {
        false
    }
    fn on_mouse_enter(&mut self, _control: &EventLoopWindowTarget<()>) -> bool {
        false
    }
    fn on_mouse_leave(&mut self, _control: &EventLoopWindowTarget<()>) -> bool {
        false
    }
    fn on_character(&mut self, _c: char, _control: &EventLoopWindowTarget<()>) -> bool {
        false
    }
    fn on_dropped_file(
        &mut self,
        _path: std::path::PathBuf,
        _control: &EventLoopWindowTarget<()>,
    ) -> bool {
        false
    }
    fn on_hover_filed(
        &mut self,
        _path: std::path::PathBuf,
        _control: &EventLoopWindowTarget<()>,
    ) -> bool {
        false
    }
    fn on_hover_canceled(&mut self, _control: &EventLoopWindowTarget<()>) -> bool {
        false
    }
    fn on_theme_changed(&mut self, _control: &EventLoopWindowTarget<()>) -> bool {
        false
    }
    fn on_scale_factor_changed(
        &mut self,
        _scale_factor: f64,
        _control: &EventLoopWindowTarget<()>,
    ) -> bool {
        false
    }
    fn on_ime(&mut self, _control: &EventLoopWindowTarget<()>) -> bool {
        false
    }
    fn on_touch(
        &mut self,
        _touch_id: u64,
        _x: i32,
        _y: i32,
        _control: &EventLoopWindowTarget<()>,
    ) -> bool {
        false
    }
    /// this method is called in all the events
    fn on(&mut self, _event: &WindowEvent, _control: &EventLoopWindowTarget<()>) -> bool {
        false
    }
    /// build the window here, with `winit::window::WindowBuilder`
    /// ## Example
    /// ```rust
    /// WindowBuilder::new()
    ///     .with_title("My window")
    /// ```
    ///
    /// check the winit docs
    fn window(&self) -> winit::window::WindowBuilder {
        winit::window::WindowBuilder::new()
    }
}
/// This macro generates a event_loop with the application handle given
/// ## Example with pollster (recommended)
/// ```rust
/// pollster::block_on(exec!(MyNewApp::new(), RendererBuilder::new()))
/// ```
#[macro_export]
macro_rules! exec {
    ($app:expr, $renderer_config:expr) => {async {
        use steamengine_core::windows::AppHandle;
        let mut app = $app;
        let event_loop = steamengine_core::winit::event_loop::EventLoop::new()?;
        let window = app.window().build(&event_loop)?;
        let mut renderer = $renderer_config.build(&window).await?;
        app.setup(&renderer)?;
        let mut surface_configured = false;
        let mut cursor_position = steamengine_core::winit::dpi::PhysicalPosition::<f64>::new(0.0, 0.0);

        event_loop
            .run(move |event, control_flow| match event {
                steamengine_core::winit::event::Event::WindowEvent {
                    ref event,
                    window_id: _,
                } => {
                    if app.on(event, control_flow) {
                        return;
                    }

                    match event {
                        steamengine_core::winit::event::WindowEvent::CloseRequested => {
                            app.on_close(control_flow);
                            control_flow.exit();
                        }
                        steamengine_core::winit::event::WindowEvent::Resized(size) => {
                            surface_configured = true;
                            if app.on_resize(size, &mut renderer, control_flow) {
                                return;
                            }
                        }
                        steamengine_core::winit::event::WindowEvent::RedrawRequested => {
                            renderer.window().request_redraw();
                            if !surface_configured {
                                return;
                            }
                            app.update(control_flow, &renderer).unwrap();
                            app.redraw(&renderer, control_flow).unwrap();
                        }
                        steamengine_core::winit::event::WindowEvent::MouseInput { state, button, .. } => {
                            let x = cursor_position.x as i32;
                            let y = cursor_position.y as i32;
                            match state {
                                steamengine_core::winit::event::ElementState::Pressed => {
                                    if app.on_mouse_down(*button, x, y, control_flow) {
                                        return;
                                    }
                                },
                                steamengine_core::winit::event::ElementState::Released => {
                                    if app.on_mouse_up(*button, x, y, control_flow) {
                                        return;
                                    }
                                    // Consider this a click when mouse is released
                                    if app.on_mouse_click(*button, x, y, control_flow) {
                                        return;
                                    }

                                    // For double-click handling, you'd need to implement timing logic
                                    // This is a simplified approach for demonstration
                                    if app.on_mouse_double_click(*button, x, y, control_flow) {
                                        return;
                                    }
                                },
                            }
                        }
                        steamengine_core::winit::event::WindowEvent::CursorMoved { position, .. } => {
                            cursor_position = *position;
                            let x = position.x as i32;
                            let y = position.y as i32;
                            if app.on_mouse_move(x, y, control_flow) {
                                return;
                            }
                        }
                        steamengine_core::winit::event::WindowEvent::KeyboardInput { event, .. } => {
                            if app.on_keyboard(event.logical_key.clone(), event.state, event, control_flow) {
                                return;
                            }
                            match event.state {
                                steamengine_core::winit::event::ElementState::Pressed => {
                                    if app.on_key_pressed(event.logical_key.clone(), control_flow) {
                                        return;
                                    }
                                },
                                steamengine_core::winit::event::ElementState::Released => {
                                    if app.on_key_released(event.logical_key.clone(), control_flow) {
                                        return;
                                    }
                                },
                            }
                        }
                        steamengine_core::winit::event::WindowEvent::Focused(focused) => {
                            if *focused {
                                if app.on_focus_gained(control_flow) {
                                    return;
                                }
                            } else {
                                if app.on_focus_lost(control_flow) {
                                    return;
                                }
                            }
                        }
                        steamengine_core::winit::event::WindowEvent::CursorEntered { .. } => {
                            if app.on_mouse_enter(control_flow) {
                                return;
                            }
                        }
                        steamengine_core::winit::event::WindowEvent::CursorLeft { .. } => {
                            if app.on_mouse_leave(control_flow) {
                                return;
                            }
                        }
                        steamengine_core::winit::event::WindowEvent::MouseWheel { delta, .. } => {
                            match delta {
                                steamengine_core::winit::event::MouseScrollDelta::LineDelta(_, y) => {
                                    if app.on_mouse_wheel(*y, control_flow) {
                                        return;
                                    }
                                },
                                steamengine_core::winit::event::MouseScrollDelta::PixelDelta(delta) => {
                                    if app.on_mouse_wheel(delta.y as f32, control_flow) {
                                        return;
                                    }
                                },
                            }
                        }
                        steamengine_core::winit::event::WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                            if app.on_scale_factor_changed(*scale_factor, control_flow) {
                                return;
                            }
                        }
                        steamengine_core::winit::event::WindowEvent::ThemeChanged(..) => {
                            if app.on_theme_changed(control_flow) {
                                return;
                            }
                        }
                        steamengine_core::winit::event::WindowEvent::DroppedFile(path) => {
                            if app.on_dropped_file(path.clone(), control_flow) {
                                return;
                            }
                        }
                        steamengine_core::winit::event::WindowEvent::HoveredFile(path) => {
                            if app.on_hover_filed(path.clone(), control_flow) {
                                return;
                            }
                        }
                        steamengine_core::winit::event::WindowEvent::HoveredFileCancelled => {
                            if app.on_hover_canceled(control_flow) {
                                return;
                            }
                        }
                        steamengine_core::winit::event::WindowEvent::Touch(touch) => {
                            let x = touch.location.x as i32;
                            let y = touch.location.y as i32;
                            if app.on_touch(touch.id, x, y, control_flow) {
                                return;
                            }
                        }
                        steamengine_core::winit::event::WindowEvent::Ime(..) => {
                            if app.on_ime(control_flow) {
                                return;
                            }
                        }
                        // For Minimized, Maximized, Restored states - in Winit 0.29 these might be handled differently
                        // through WindowEvent::WindowFlags if it exists
                        _ => {
                            // For any other events
                        }
                    }
                }
                steamengine_core::winit::event::Event::AboutToWait => {
                    // This could be used for continuous rendering, if needed
                    renderer.window().request_redraw();
                }
                _ => (),
            })?;
    Ok(()) as Result<(), steamengine_core::windows::errors::AppError>
    }};
}

use winit::{
    event::{MouseButton, WindowEvent},
    keyboard::Key,
};

pub trait AppHandle {
    fn redraw(&mut self);
    fn update(&mut self);
    fn on_close(&mut self) -> bool {
        false
    }
    fn on_resize(&mut self, _size: &winit::dpi::PhysicalSize<u32>) -> bool {
        false
    }
    fn on_mouse_move(&mut self, _x: i32, _y: i32) -> bool {
        false
    }
    fn on_mouse_down(&mut self, _button: MouseButton, _x: i32, _y: i32) -> bool {
        false
    }
    fn on_mouse_up(&mut self, _button: MouseButton, _x: i32, _y: i32) -> bool {
        false
    }
    fn on_mouse_click(&mut self, _button: MouseButton, _x: i32, _y: i32) -> bool {
        false
    }
    fn on_mouse_double_click(&mut self, _button: MouseButton, _x: i32, _y: i32) -> bool {
        false
    }
    fn on_minimize(&mut self) -> bool {
        false
    }
    fn on_maximize(&mut self) -> bool {
        false
    }
    fn on_restore(&mut self) -> bool {
        false
    }
    fn on_fullscreen(&mut self) -> bool {
        false
    }
    fn on_exit_fullscreen(&mut self) -> bool {
        false
    }
    fn on_keyboard(&mut self, _key: Key) -> bool {
        false
    }
    fn on_key_pressed(&mut self, _key: Key) -> bool {
        false
    }
    fn on_key_released(&mut self, _key: Key) -> bool {
        false
    }
    fn on_focus_gained(&mut self) -> bool {
        false
    }
    fn on_focus_lost(&mut self) -> bool {
        false
    }
    fn on_mouse_wheel(&mut self, _delta: f32) -> bool {
        false
    }
    fn on_mouse_enter(&mut self) -> bool {
        false
    }
    fn on_mouse_leave(&mut self) -> bool {
        false
    }
    fn on_character(&mut self, _c: char) -> bool {
        false
    }
    fn on_dropped_file(&mut self, _path: std::path::PathBuf) -> bool {
        false
    }
    fn on_hover_filed(&mut self, _path: std::path::PathBuf) -> bool {
        false
    }
    fn on_hover_canceled(&mut self) -> bool {
        false
    }
    fn on_theme_changed(&mut self) -> bool {
        false
    }
    fn on_scale_factor_changed(&mut self, _scale_factor: f64) -> bool {
        false
    }
    fn on_ime(&mut self) -> bool {
        false
    }
    fn on_touch(&mut self, _touch_id: u64, _x: i32, _y: i32) -> bool {
        false
    }
    fn on(&mut self, _event: &WindowEvent) -> bool {
        false
    }
    fn window(&self) -> winit::window::WindowBuilder {
        winit::window::WindowBuilder::new()
    }
}
/// This macro generates a event_loop with the application handle given
#[macro_export]
macro_rules! exec {
    ($app:expr) => {{
        let mut app = $app;
        let event_loop = winit::event_loop::EventLoop::new().unwrap();
        let window = app.window().build(&event_loop).unwrap();
        let mut surface_configured = false;
        let mut cursor_position = winit::dpi::PhysicalPosition::<f64>::new(0.0, 0.0);

        event_loop
            .run(move |event, control_flow| match event {
                winit::event::Event::WindowEvent {
                    ref event,
                    window_id: _,
                } => {
                    if app.on(event) {
                        return;
                    }

                    match event {
                        winit::event::WindowEvent::CloseRequested => {
                            if app.on_close() {
                                return;
                            }
                            control_flow.exit();
                        }
                        winit::event::WindowEvent::Resized(size) => {
                            surface_configured = true;
                            if app.on_resize(size) {
                                return;
                            }
                        }
                        winit::event::WindowEvent::RedrawRequested => {
                            window.request_redraw();
                            if !surface_configured {
                                return;
                            }
                            app.update();
                            app.redraw();
                            // TODO: Manage redraw errors
                        }
                        winit::event::WindowEvent::MouseInput { state, button, .. } => {
                            let x = cursor_position.x as i32;
                            let y = cursor_position.y as i32;
                            match state {
                                winit::event::ElementState::Pressed => {
                                    if app.on_mouse_down(*button, x, y) {
                                        return;
                                    }
                                },
                                winit::event::ElementState::Released => {
                                    if app.on_mouse_up(*button, x, y) {
                                        return;
                                    }
                                    // Consider this a click when mouse is released
                                    if app.on_mouse_click(*button, x, y) {
                                        return;
                                    }

                                    // For double-click handling, you'd need to implement timing logic
                                    // This is a simplified approach for demonstration
                                    if app.on_mouse_double_click(*button, x, y) {
                                        return;
                                    }
                                },
                            }
                        }
                        winit::event::WindowEvent::CursorMoved { position, .. } => {
                            cursor_position = *position;
                            let x = position.x as i32;
                            let y = position.y as i32;
                            if app.on_mouse_move(x, y) {
                                return;
                            }
                        }
                        winit::event::WindowEvent::KeyboardInput { event, .. } => {
                            if app.on_keyboard(event.logical_key.clone()) {
                                return;
                            }
                            match event.state {
                                winit::event::ElementState::Pressed => {
                                    if app.on_key_pressed(event.logical_key.clone()) {
                                        return;
                                    }
                                },
                                winit::event::ElementState::Released => {
                                    if app.on_key_released(event.logical_key.clone()) {
                                        return;
                                    }
                                },
                            }
                        }
                        winit::event::WindowEvent::Focused(focused) => {
                            if *focused {
                                if app.on_focus_gained() {
                                    return;
                                }
                            } else {
                                if app.on_focus_lost() {
                                    return;
                                }
                            }
                        }
                        winit::event::WindowEvent::CursorEntered { .. } => {
                            if app.on_mouse_enter() {
                                return;
                            }
                        }
                        winit::event::WindowEvent::CursorLeft { .. } => {
                            if app.on_mouse_leave() {
                                return;
                            }
                        }
                        winit::event::WindowEvent::MouseWheel { delta, .. } => {
                            match delta {
                                winit::event::MouseScrollDelta::LineDelta(_, y) => {
                                    if app.on_mouse_wheel(*y) {
                                        return;
                                    }
                                },
                                winit::event::MouseScrollDelta::PixelDelta(delta) => {
                                    if app.on_mouse_wheel(delta.y as f32) {
                                        return;
                                    }
                                },
                            }
                        }
                        winit::event::WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                            if app.on_scale_factor_changed(*scale_factor) {
                                return;
                            }
                        }
                        winit::event::WindowEvent::ThemeChanged(..) => {
                            if app.on_theme_changed() {
                                return;
                            }
                        }
                        winit::event::WindowEvent::DroppedFile(path) => {
                            if app.on_dropped_file(path.clone()) {
                                return;
                            }
                        }
                        winit::event::WindowEvent::HoveredFile(path) => {
                            if app.on_hover_filed(path.clone()) {
                                return;
                            }
                        }
                        winit::event::WindowEvent::HoveredFileCancelled => {
                            if app.on_hover_canceled() {
                                return;
                            }
                        }
                        winit::event::WindowEvent::Touch(touch) => {
                            let x = touch.location.x as i32;
                            let y = touch.location.y as i32;
                            if app.on_touch(touch.id, x, y) {
                                return;
                            }
                        }
                        winit::event::WindowEvent::Ime(..) => {
                            if app.on_ime() {
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
                winit::event::Event::AboutToWait => {
                    // This could be used for continuous rendering, if needed
                    window.request_redraw();
                }
                _ => (),
            })
            .unwrap();
    }};
}

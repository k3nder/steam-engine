use cgmath::*;
use steamengine_renderer_util::camera::Camera;
use tracing::*;
use winit::event::ElementState;
use winit::event::KeyEvent;
use winit::event::WindowEvent;
use winit::keyboard::KeyCode;
use winit::keyboard::PhysicalKey;

pub struct CameraController {
    speed: f32,
    is_forward_pressed: bool,
    is_backward_pressed: bool,
    is_left_pressed: bool,
    is_right_pressed: bool,
}

impl CameraController {
    pub fn new(speed: f32) -> Self {
        Self {
            speed,
            is_forward_pressed: false,
            is_backward_pressed: false,
            is_left_pressed: false,
            is_right_pressed: false,
        }
    }

    pub fn process_events(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        state,
                        physical_key: PhysicalKey::Code(keycode),
                        ..
                    },
                ..
            } => {
                let is_pressed = *state == ElementState::Pressed;
                match keycode {
                    KeyCode::KeyW | KeyCode::ArrowUp => {
                        debug!("W");
                        self.is_forward_pressed = is_pressed;
                        true
                    }
                    KeyCode::KeyA | KeyCode::ArrowLeft => {
                        debug!("A");
                        self.is_left_pressed = is_pressed;
                        true
                    }
                    KeyCode::KeyS | KeyCode::ArrowDown => {
                        debug!("S");
                        self.is_backward_pressed = is_pressed;
                        true
                    }
                    KeyCode::KeyD | KeyCode::ArrowRight => {
                        debug!("D");
                        self.is_right_pressed = is_pressed;
                        true
                    }
                    _ => false,
                }
            }
            _ => false,
        }
    }

    pub fn update_camera(&self, camera: &mut impl Camera) {
        use cgmath::InnerSpace;
        let forward = camera.target().clone().to_vec() - camera.eye().clone().to_vec();
        let forward_norm = forward.normalize();
        let forward_mag = forward.magnitude();

        // Prevents glitching when the camera gets too close to the
        // center of the scene.
        if self.is_forward_pressed && forward_mag > self.speed {
            *camera.eye() += forward_norm * self.speed;
        }
        if self.is_backward_pressed {
            *camera.eye() -= forward_norm * self.speed;
        }

        let right = forward_norm.cross(camera.up().clone());

        // Redo radius calc in case the forward/backward is pressed.
        let forward = camera.target().clone().to_vec() - camera.eye().clone().to_vec();
        let forward_mag = forward.magnitude();

        if self.is_right_pressed {
            // Rescale the distance between the target and the eye so
            // that it doesn't change. The eye, therefore, still
            // lies on the circle made by the target and eye.
            *camera.eye() = Point3::from_vec(
                camera.target().clone().to_vec()
                    - (forward + right * self.speed).normalize() * forward_mag,
            );
        }
        if self.is_left_pressed {
            *camera.eye() = Point3::from_vec(
                camera.target().clone().to_vec()
                    - (forward - right * self.speed).normalize() * forward_mag,
            );
        }
    }
}

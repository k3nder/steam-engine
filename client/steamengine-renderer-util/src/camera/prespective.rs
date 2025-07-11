use super::Camera;
use glam::*;

/// Implementation of camera for Prespective Camera
/// ## Example
/// ```rust
/// use cgmath::point3;
/// use steamengine_renderer_util::camera::create_bindings;
/// use steamengine_renderer_util::camera::CameraBuffer;
///
/// let camera = PrespectiveCamera::default();
/// // set the aspect ratio with window properties
/// *camera.aspect_ratio() = width / height;
/// // sets the camera location
/// *camera.eye() = point3(0.0, 0.0, -2.0);
///
/// // upload the camera to the shader
/// let buffer = CameraBuffer::new(&renderer, 1);
/// let binding = create_bindings(&renderer, buffer.as_entrie());
///
/// buffer.set(0, camera);
///
/// ```

#[derive(Clone, Debug)]
pub struct PrespectiveCamera {
    // view matrix
    eye: Vec3,
    target: Vec3,
    up: Vec3,

    // projection
    fov: f32,
    aspect_ratio: f32,
    near: f32,
    far: f32,
}
impl PrespectiveCamera {
    pub fn fov(&mut self) -> &mut f32 {
        &mut self.fov
    }
    pub fn aspect_ratio(&mut self) -> &mut f32 {
        &mut self.aspect_ratio
    }
    pub fn near(&mut self) -> &mut f32 {
        &mut self.near
    }
    pub fn far(&mut self) -> &mut f32 {
        &mut self.far
    }
}
impl Default for PrespectiveCamera {
    fn default() -> Self {
        let eye = vec3(0.0, 1.0, 2.0);
        let target = vec3(0.0, 0.0, 0.0);
        let up = vec3(0.0, 1.0, 0.0);

        let fov = 45.0;
        let aspect_ratio = 16.0 / 9.0;
        let near = 0.1;
        let far = 100.0;

        Self {
            eye,
            target,
            up,
            fov,
            aspect_ratio,
            near,
            far,
        }
    }
}

impl Camera for PrespectiveCamera {
    fn up(&mut self) -> &mut Vec3 {
        &mut self.up
    }
    fn eye(&mut self) -> &mut Vec3 {
        &mut self.eye
    }
    fn target(&mut self) -> &mut Vec3 {
        &mut self.target
    }
    fn view(&self) -> Mat4 {
        Mat4::look_at_rh(self.eye, self.target, self.up)
    }
    fn projection(&self) -> Mat4 {
        Mat4::perspective_lh(self.fov, self.aspect_ratio, self.near, self.far)
    }
}

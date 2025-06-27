use super::Camera;
use cgmath::Matrix4;
use cgmath::Point3;
use cgmath::Vector3;
use cgmath::ortho;

/// Implementation of camera for Orthographic Camera
/// ## Example
/// ```rust
/// use cgmath::point3;
/// use steamengine_renderer_util::camera::create_bindings;
/// use steamengine_renderer_util::camera::CameraBuffer;
///
/// let camera = OrthographicCamera::default();
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
pub struct OrthographicCamera {
    // view matrix
    eye: Point3<f32>,
    target: Point3<f32>,
    up: Vector3<f32>,

    left: f32,
    right: f32,
    bottom: f32,
    upper: f32,
    near: f32,
    far: f32,
}
impl OrthographicCamera {
    pub fn default() -> Self {
        let eye = Point3::new(0.0, 0.0, 5.0);
        let target = Point3::new(0.0, 0.0, 0.0);
        let up = Vector3::new(0.0, 1.0, 0.0);

        let left = -10.0;
        let right = 10.0;
        let bottom = -10.0;
        let upper = 10.0;
        let near = 0.1;
        let far = 100.0;
        Self {
            eye,
            target,
            up,
            left,
            right,
            bottom,
            upper,
            near,
            far,
        }
    }
}
impl Camera for OrthographicCamera {
    fn up(&mut self) -> &mut Vector3<f32> {
        &mut self.up
    }
    fn eye(&mut self) -> &mut Point3<f32> {
        &mut self.eye
    }
    fn target(&mut self) -> &mut Point3<f32> {
        &mut self.target
    }
    fn view(&self) -> Matrix4<f32> {
        Matrix4::look_at_rh(self.eye, self.target, self.up)
    }
    fn projection(&self) -> Matrix4<f32> {
        ortho(
            self.left,
            self.right,
            self.bottom,
            self.upper,
            self.near,
            self.far,
        )
    }
}

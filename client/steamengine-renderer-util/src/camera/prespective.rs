use super::Camera;
use cgmath::Deg;
use cgmath::Matrix4;
use cgmath::Point3;
use cgmath::Vector3;
use cgmath::perspective;

#[derive(Clone, Debug)]
pub struct PrespectiveCamera {
    // view matrix
    eye: Point3<f32>,
    target: Point3<f32>,
    up: Vector3<f32>,

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
        let eye = Point3::new(0.0, 1.0, 2.0);
        let target = Point3::new(0.0, 0.0, 0.0);
        let up = Vector3::unit_y();

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
        perspective(Deg(self.fov), self.aspect_ratio, self.near, self.far)
    }
}

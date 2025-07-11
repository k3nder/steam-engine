use glam::*;

/// Use and create cameras easily

/// Implementation for orthographic camera
/// Enable it with feature "orthographic-camera"
#[cfg(feature = "orthographic-camera")]
pub mod orthographic;
/// Implementation for prespective camera
/// Enable it with featurer "prespective-camera"
#[cfg(feature = "prespective-camera")]
pub mod prespective;

/// Abstraction of a camera
pub trait Camera: Send + Sync + Clone {
    /// Create a view matrix for the camera
    fn view(&self) -> Mat4;
    /// Create a projection matrix
    fn projection(&self) -> Mat4;
    /// Poistion of the camera
    fn eye(&mut self) -> &mut Vec3;
    /// Direction of the camera
    fn target(&mut self) -> &mut Vec3;
    /// Up vector
    fn up(&mut self) -> &mut Vec3;
    /// Calculated and converted to WGPU matrix of the camera
    fn matrix(&self) -> Mat4 {
        let projection = self.projection();
        let view = self.view();
        let matrix = OPENGL_TO_WGPU_MATRIX * projection * view;
        matrix
    }
}

/// Constant for conversion of opengl camera to wgpu camera
#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: Mat4 = Mat4::from_cols(
    vec4(1.0, 0.0, 0.0, 0.0),
    vec4(0.0, 1.0, 0.0, 0.0),
    vec4(0.0, 0.0, 0.5, 0.0),
    vec4(0.0, 0.0, 0.5, 1.0),
);

/// Simple buffer Implementation for camera matrix
#[cfg(feature = "simple-buffers")]
pub struct CameraBuffer<'a> {
    buffer: wgpu::Buffer,
    renderer: std::sync::Arc<steamengine_renderer::Renderer<'a>>,
    limit: u64,
}
#[cfg(feature = "simple-buffers")]
impl<'a> crate::simple_buffer::SimpleBuffer<'a, [[f32; 4]; 4]> for CameraBuffer<'a> {
    fn new(renderer: std::sync::Arc<steamengine_renderer::Renderer<'a>>, limit: u64) -> Self {
        let lock = renderer.clone();
        let buffer = lock.create_buffer(
            "Camera buffer",
            wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            size_of::<[[f32; 4]; 4]>() as u64 * limit,
        );
        Self {
            renderer,
            buffer,
            limit,
        }
    }
    fn as_entrie(&self) -> wgpu::BindingResource {
        self.buffer.as_entire_binding()
    }
    fn buffer(&self) -> &wgpu::Buffer {
        &self.buffer
    }
    fn limit(&self) -> u64 {
        self.limit
    }
    fn renderer(&self) -> std::sync::Arc<steamengine_renderer::Renderer<'a>> {
        self.renderer.clone()
    }
}
#[cfg(feature = "simple-buffers")]
impl CameraBuffer<'_> {
    pub fn set_camera<T: Camera>(&self, camera: T) {
        use crate::simple_buffer::SimpleBuffer;
        let matrix: [[f32; 4]; 4] = camera.matrix().to_cols_array_2d();
        self.set(0, matrix);
    }
}

/// Function to create default bindings for a camera
/// Create a bind group with label `Camera bindings` and a entry in binding *0* that contrains the
/// matrix accesible in *vertex* shader stage
#[cfg(feature = "simple-bindings")]
pub fn create_bindings(
    renderer: std::sync::Arc<steamengine_renderer::Renderer>,
    buffer: wgpu::BindingResource,
) -> crate::bindings::Bindings {
    use crate::bindings::CreateBindings;
    use steamengine_renderer::bind_group::BindGroupEntryBuilder;
    let bindings = renderer.new_bindings(
        "Camera Bindings",
        &[BindGroupEntryBuilder::new(0)
            .uniform()
            .with(buffer)
            .on(wgpu::ShaderStages::VERTEX)],
    );
    bindings
}

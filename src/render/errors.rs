use image::ImageError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum TextureError {
    #[error("Error loading texture")]
    Loading(#[from] ImageError),
    #[error("Error reading texture")]
    Reading(#[from] std::io::Error),
}

#[derive(Debug, Error)]
pub enum RenderError {
    #[error("Failed to setup renderer, {0}")]
    RendererSetup(#[from] RendererSetupError),
}

#[derive(Debug, Error)]
pub enum RendererSetupError {
    #[error("Failed to create surface, {0}")]
    SurfaceCreation(#[from] wgpu::CreateSurfaceError),
    #[error("Failed to get adapter, {0}")]
    AdapterRequest(#[from] wgpu::RequestAdapterError),
    #[error("Failed to get device, {0}")]
    DeviceRequest(#[from] wgpu::RequestDeviceError),
}

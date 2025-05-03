/// This module contrains funcions to manage threads
#[macro_use]
pub mod threads;
/// This module contrains utilites to build a app
#[macro_use]
pub mod windows;
/// This module contrains an api to comunicate to WGPU
#[macro_use]
pub mod render;

pub use bytemuck;
pub use wgpu;
pub use winit;

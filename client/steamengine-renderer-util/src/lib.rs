/// Module with camea utils
/// Contrais implementations of trait camera
/// Enable it with feature "camera"
#[cfg(feature = "camera")]
pub mod camera;

/// Module with resource managers
/// Load resources of diferent types in memory
/// Enable it with feature "resource-manager"
#[cfg(feature = "resource-manager")]
pub mod resources;

/// Module with simple buffers
/// A way to use buffers more easy
/// Enable it with feature "simple-buffers"
#[cfg(feature = "simple-buffers")]
pub mod simple_buffer;

/// Module with bindings
/// Group a bind_group and layout into a single structure
/// Enable it with feature "simple-bindings"
#[cfg(feature = "simple-bindings")]
pub mod bindings;

/// Module with depth-textures
/// Create depth textures easily
/// Enable it with feature "depth-textures"
#[cfg(feature = "depth-textures")]
pub mod depth_texture;

/// Module with errors
pub mod errors;

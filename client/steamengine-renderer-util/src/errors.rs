use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("io error")]
    IOError(#[from] std::io::Error),
    #[cfg(feature = "resource-manager")]
    #[error("fs extra error")]
    FSError(#[from] fs_extra::error::Error),
    #[cfg(feature = "texture-resource-manager")]
    #[error("image load error")]
    ImageError(#[from] image::error::ImageError),
    #[error("from UTF8 error")]
    Utf8Error(#[from] std::string::FromUtf8Error),
    #[cfg(feature = "model-resource-manager")]
    #[error("model load error")]
    ModelLoadError(#[from] tobj::LoadError),
}

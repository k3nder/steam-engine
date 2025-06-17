use thiserror::Error;
#[cfg(feature = "tokio-comp")]
use tokio::sync::mpsc::error::{SendError, TryRecvError};
#[cfg(feature = "tokio-comp")]
use crate::Package;

#[derive(Debug, Error)]
pub enum PCSError {
    #[error("IOError")]
    IOError(#[from] std::io::Error),
    #[cfg(feature = "tokio-comp")]
    #[error("Error trying to receive a package")]
    TryRecvError(#[from] TryRecvError),
    #[cfg(feature = "tokio-comp")]
    #[error("Error sending a package")]
    SendError(#[from] SendError<Package>)
}

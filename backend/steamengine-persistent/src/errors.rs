use redis::RedisError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum VKSError {
    #[error("Error in redis connection")]
    RedisError(#[from] RedisError),
    #[error("Rkyv error")]
    RKYVError(#[from] RkyvError)
}

#[derive(Debug)]
pub struct RkyvError(pub Box<dyn std::any::Any + Send + 'static>);

impl std::fmt::Display for RkyvError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "error interno de rkyv")
    }
}

impl std::error::Error for RkyvError {}

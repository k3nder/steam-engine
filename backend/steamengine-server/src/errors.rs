use redis::RedisError;
use thiserror::Error;
use tokio::task::JoinError;

#[derive(Error, Debug)]
pub enum ServerError {
    #[error("IOError")]
    IOError(#[from] std::io::Error),
    #[error("Join error")]
    JoinError(#[from] JoinError),
    #[error("Error on redis")]
    RedisError(#[from] RedisError),
}

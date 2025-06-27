use std::{collections::HashMap, time::Instant};

use crate::Persistent;
use redis::{Commands, Connection, RedisError, TypedCommands};
use rkyv::{
    Archive, Deserialize, Serialize,
    api::high::{HighSerializer, HighValidator, to_bytes_with_alloc},
    bytecheck::CheckBytes,
    de::Pool,
    rancor::{Source, Strategy},
    ser::allocator::ArenaHandle,
    util::AlignedVec,
};
use tracing::{debug, trace};

impl Persistent for Connection {
    type Error = VKError;

    fn index(&mut self, key: &str) -> Result<isize, Self::Error> {
        trace!("Increment index of key {}", key);
        let new = TypedCommands::incr(self, key, 1)?;
        Ok(new)
    }
    fn create_entity(&mut self, components: &[&str]) -> Result<isize, Self::Error> {
        trace!("Creating new entity with {} components", components.len());
        let index = self.index("entities:len")?;
        trace!("New entity with id {}", index);
        let key = format!("entities:{}", index);
        trace!("Pushing into persistent DB");
        debug!(
            "Pushing new entity with {} components and key {}",
            components.len(),
            key
        );
        let time = Instant::now();
        TypedCommands::lpush(self, &key, components)?;
        TypedCommands::hset(self, "entities", &index, &key)?;
        debug!("Entity pushed in {}ms", time.elapsed().as_millis());
        Ok(index)
    }
    fn create_component<'a, E: Source>(
        &mut self,
        typ: &str,
        component: impl for<'b> Serialize<HighSerializer<AlignedVec, ArenaHandle<'b>, E>> + Send,
        arena: ArenaHandle<'a>,
    ) -> Result<String, Self::Error> {
        trace!("Creating new component of type {}", typ);
        let key = format!("cdata:{}", typ);
        debug!("Serializing component");
        let time = Instant::now();
        let buffer = rkyv::api::high::to_bytes_with_alloc(&component, arena)
            .map_err(|e| VKError::RKYVError(RkyvError(Box::new(e))))?;
        debug!("Component serialized in {}ms", time.elapsed().as_millis());
        let len_key = format!("{}:len", key);
        let digest = self.index(&len_key)?;
        trace!("Pushing component into DB");
        TypedCommands::hset(self, &key, digest, buffer.as_slice())?;
        Ok(format!("{}:{}", typ, digest))
    }

    fn components_of_type(&mut self, typ: &str) -> Result<HashMap<String, Vec<u8>>, Self::Error> {
        let key = format!("cdata:{}", typ);
        let map: HashMap<String, Vec<u8>> = Commands::hgetall(self, key)?;
        Ok(map)
    }

    fn get_entity(&mut self, id: isize) -> Result<Vec<String>, Self::Error> {
        let key = format!("entities:{}", id);
        Ok(TypedCommands::lrange(self, key, 0, -1)?)
    }

    fn get_component<T, E>(&mut self, id: &str) -> Result<T, Self::Error>
    where
        T: Archive,
        T::Archived: for<'a> CheckBytes<HighValidator<'a, E>> + Deserialize<T, Strategy<Pool, E>>,
        E: Source,
    {
        let split: Vec<&str> = id.split(":").collect();
        let typ = split[0];
        let id = split[1];
        let key = format!("cdata:{}", typ);
        let buffer: Vec<u8> = Commands::hget(self, key, id)?;
        println!("buffer {:?}", buffer);
        Ok(rkyv::from_bytes(buffer.as_slice())
            .map_err(|e| VKError::RKYVError(RkyvError(Box::new(e))))?)
    }

    fn set_component<'a, E, C>(
        &mut self,
        id: &str,
        component: C,
        arena: ArenaHandle<'a>,
    ) -> Result<(), Self::Error>
    where
        E: Source,
        C: for<'b> Serialize<HighSerializer<AlignedVec, ArenaHandle<'b>, E>> + Send,
    {
        let split: Vec<_> = id.split(":").collect();
        let typ = split[0];
        let id = split[1];
        let typ_key = format!("cdata:{}", typ);
        let buffer = to_bytes_with_alloc(&component, arena)
            .map_err(|e| Self::Error::RKYVError(RkyvError(Box::new(e))))?;
        TypedCommands::hset(self, &typ_key, id, buffer.as_slice())?;
        Ok(())
    }

    fn set_entity(&mut self, id: isize, components: &[&str]) -> Result<(), Self::Error> {
        let key = format!("entities:{}", id);
        TypedCommands::lrem(self, &key, 0, -1)?;
        TypedCommands::lpush(self, &key, components)?;
        Ok(())
    }

    fn set(&mut self, id: &str, value: Vec<u8>) -> Result<(), Self::Error> {
        Ok(Commands::set(self, id, value)?)
    }

    fn get(&mut self, id: &str) -> Result<Vec<u8>, Self::Error> {
        Ok(Commands::get(self, id)?)
    }
}

#[cfg(feature = "asyncron")]
impl crate::asyncron::AsyncPersistent for redis::aio::MultiplexedConnection {
    type Error = VKError;

    async fn index(&mut self, key: &str) -> Result<isize, Self::Error> {
        trace!("Increment index of key {}", key);
        let new = redis::AsyncTypedCommands::incr(self, key, 1).await?;
        Ok(new)
    }
    async fn create_entity(&mut self, components: &[&str]) -> Result<isize, Self::Error> {
        trace!("Creating new entity with {} components", components.len());
        let index = self.index("entities:len").await?;
        trace!("New entity with id {}", index);
        let key = format!("entities:{}", index);
        trace!("Pushing into persistent DB");
        debug!(
            "Pushing new entity with {} components and key {}",
            components.len(),
            key
        );
        let time = Instant::now();
        redis::AsyncTypedCommands::lpush(self, &key, components).await?;
        redis::AsyncTypedCommands::hset(self, "entities", &index, &key).await?;
        debug!("Entity pushed in {}ms", time.elapsed().as_millis());
        Ok(index)
    }
    async fn create_component<'a, E: Source>(
        &mut self,
        typ: &str,
        component: impl for<'b> Serialize<HighSerializer<AlignedVec, ArenaHandle<'b>, E>> + Send,
        arena: ArenaHandle<'a>,
    ) -> Result<String, Self::Error> {
        trace!("Creating new component of type {}", typ);
        let key = format!("cdata:{}", typ);
        debug!("Serializing component");
        let time = Instant::now();
        let buffer = rkyv::api::high::to_bytes_with_alloc(&component, arena)
            .map_err(|e| VKError::RKYVError(RkyvError(Box::new(e))))?;
        debug!("Component serialized in {}ms", time.elapsed().as_millis());
        let len_key = format!("{}:len", key);
        let digest = self.index(&len_key).await?;
        trace!("Pushing component into DB");
        redis::AsyncTypedCommands::hset(self, &key, digest, buffer.as_slice()).await?;
        Ok(format!("{}:{}", typ, digest))
    }

    async fn components_of_type(
        &mut self,
        typ: &str,
    ) -> Result<HashMap<String, Vec<u8>>, Self::Error> {
        let key = format!("cdata:{}", typ);
        let map: HashMap<String, Vec<u8>> = redis::AsyncCommands::hgetall(self, key).await?;
        Ok(map)
    }

    async fn get_entity(&mut self, id: isize) -> Result<Vec<String>, Self::Error> {
        let key = format!("entities:{}", id);
        Ok(redis::AsyncTypedCommands::lrange(self, key, 0, -1).await?)
    }

    async fn get_component<T, E>(&mut self, id: &str) -> Result<T, Self::Error>
    where
        T: Archive,
        T::Archived: for<'a> CheckBytes<HighValidator<'a, E>> + Deserialize<T, Strategy<Pool, E>>,
        E: Source,
    {
        let split: Vec<&str> = id.split(":").collect();
        let typ = split[0];
        let id = split[1];
        let key = format!("cdata:{}", typ);
        let buffer: Vec<u8> = redis::AsyncCommands::hget(self, key, id).await?;
        println!("buffer {:?}", buffer);
        Ok(rkyv::from_bytes(buffer.as_slice())
            .map_err(|e| VKError::RKYVError(RkyvError(Box::new(e))))?)
    }

    async fn set_component<'a, E, C>(
        &mut self,
        id: &str,
        component: C,
        arena: ArenaHandle<'a>,
    ) -> Result<(), Self::Error>
    where
        E: Source,
        C: for<'b> Serialize<HighSerializer<AlignedVec, ArenaHandle<'b>, E>> + Send,
    {
        let split: Vec<_> = id.split(":").collect();
        let typ = split[0];
        let id = split[1];
        let typ_key = format!("cdata:{}", typ);
        let buffer = to_bytes_with_alloc(&component, arena)
            .map_err(|e| Self::Error::RKYVError(RkyvError(Box::new(e))))?;
        redis::AsyncTypedCommands::hset(self, &typ_key, id, buffer.as_slice()).await?;
        Ok(())
    }

    async fn set_entity(&mut self, id: isize, components: &[&str]) -> Result<(), Self::Error> {
        let key = format!("entities:{}", id);
        redis::AsyncTypedCommands::lrem(self, &key, 0, -1).await?;
        redis::AsyncTypedCommands::lpush(self, &key, components).await?;
        Ok(())
    }

    async fn set(&mut self, id: &str, value: Vec<u8>) -> Result<(), Self::Error> {
        Ok(redis::AsyncCommands::set(self, id, value).await?)
    }

    async fn get(&mut self, id: &str) -> Result<Vec<u8>, Self::Error> {
        Ok(redis::AsyncCommands::get(self, id).await?)
    }
}

#[derive(thiserror::Error, Debug)]
pub enum VKError {
    #[error("Error in redis connection")]
    RedisError(#[from] RedisError),
    #[error("Rkyv error")]
    RKYVError(#[from] RkyvError),
}

#[derive(Debug)]
pub struct RkyvError(pub Box<dyn std::any::Any + Send + 'static>);

impl std::fmt::Display for RkyvError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "error interno de rkyv")
    }
}

impl std::error::Error for RkyvError {}

use std::{collections::HashMap, time::Instant};

use redis::{aio::{MultiplexedConnection}, AsyncCommands, AsyncTypedCommands, RedisError};
use rkyv::{api::{access_pos_unchecked, high::{to_bytes_with_alloc, HighSerializer, HighValidator}, root_position}, bytecheck::CheckBytes, de::Pool, rancor::{Source, Strategy}, ser::allocator::ArenaHandle, util::AlignedVec, Archive, Deserialize, Serialize};

use tracing::{debug, trace};

use crate::errors::{RkyvError, VKSError};
/// Module with errors
pub mod errors;

pub trait VKSPersistent {
    /// function to obtain an index that grows for each call.
    /// ## Example
    /// ```rust
    /// let index = vks.index("counter")?;
    /// assert_eq!(index, 1);
    /// let index = vks.index("counter")?;
    /// assert_eq!(index, 2);
    /// ```
    fn index(&mut self, key: &str) -> impl Future<Output = Result<isize, RedisError>>;
    /// function that generates a new entity with the provided components, returns the id of the entity
    /// ## Example
    /// ```rust
    /// let entity = vks.create_entity(&["render:13", "clock:45"])?;
    /// ```
    fn create_entity(&mut self, components: &[&str]) -> impl Future<Output = Result<isize, VKSError>>;
    /// uploads a new component to the database, converting the component to binary with rkyv, returns the component id
    /// ## Example
    /// ```rust
    /// use rkyv::{rancor::Failure, Archive, Deserialize, Serialize};
    ///
    /// #[derive(Serialize, Deserialize, Archive, Debug)]
    /// struct ClockComponent {
    ///     pub time: usize
    /// };
    /// // rkyv arena
    /// let mut arena = Arena::new();
    /// let id = vks.create_component::<Failure>(&ClockComponent {
    ///     time: 1
    /// }, arena.aquerie())?;
    /// ```
    fn create_component<'a, E: Source>(
        &mut self,
        typ: &str,
        component: impl for<'b> Serialize<HighSerializer<AlignedVec, ArenaHandle<'b>, E>>,
        arena: ArenaHandle<'a>
    ) -> impl Future<Output = Result<String, VKSError>>;
    /// obtains all the components of a specific type
    /// ## Example
    /// ```rust
    /// let clocks = vks.components_of_type("clock")?;
    /// ```
    fn components_of_type(&mut self, typ: &str) -> impl Future<Output = Result<HashMap<String, Vec<u8>>, VKSError>>;
    /// gets the components of an entity with its id
    /// ## Example
    /// ```rust
    /// let components = vks.get_entity("entities:1")?;
    /// ```
    fn get_entity(&mut self, id: isize) -> impl Future<Output = Result<Vec<String>, VKSError>>;
    /// Gets the data of a specific component with its id
    /// ## Example
    /// ```rust
    /// let component = vks.get_component("clock:13");
    /// ```
    fn get_component<T, E>(&mut self, id: &str) -> impl Future<Output = Result<T, VKSError>>
    where
        T: Archive,
        T::Archived: for<'a> CheckBytes<HighValidator<'a, E>>
            + Deserialize<T, Strategy<Pool, E>>,
        E: Source;
    /// Sets the data of a specific component with its id
    /// ## Example
    /// ```rust
    /// // rkyv arena
    /// let mut arena = Arena::new();
    /// vks.set_component("clock:13", ClockComponent {
    ///     time: 2usize
    /// }, arena.aquerie())?;
    /// ```
    fn set_component<
        'a,
        E: Source
    >(
        &mut self,
        id: &str,
        component: &impl for<'b> Serialize<
            HighSerializer<AlignedVec, ArenaHandle<'b>, E>,
        >,
        arena: ArenaHandle<'a>
    ) -> impl Future<Output=Result<(), VKSError>>;
    /// Replaces the data of an entity by other data
    /// ## Example
    /// ```rust
    /// vks.set_entity(2, &["clock:55"])
    /// ```
    fn set_entity(&mut self, id: isize, components: &[&str]) -> impl Future<Output=Result<(), VKSError>>;
}
impl VKSPersistent for MultiplexedConnection {
    async fn index(&mut self, key: &str) -> Result<isize, RedisError> {
        trace!("Increment index of key {}", key);
        let new = AsyncTypedCommands::incr(self, key, 1).await?;
        Ok(new)
    }
    async fn create_entity(&mut self, components: &[&str]) -> Result<isize, VKSError> {
        trace!("Creating new entity with {} components", components.len());
        let index = self.index("entities:len").await?;
        trace!("New entity with id {}", index);
        let key = format!("entities:{}", index);
        trace!("Pushing into persistent DB");
        debug!("Pushing new entity with {} components and key {}", components.len(), key);
        let time = Instant::now();
        AsyncTypedCommands::lpush(self, &key, components).await?;
        AsyncTypedCommands::hset(self, "entities", &index, &key).await?;
        debug!("Entity pushed in {}ms", time.elapsed().as_millis());
        Ok(index)
    }
    async fn create_component<'a, E: Source>(
        &mut self,
        typ: &str,
        component: impl for<'b> Serialize<HighSerializer<AlignedVec, ArenaHandle<'b>, E>>,
        arena: ArenaHandle<'a>
    ) -> Result<String, VKSError> {
        trace!("Creating new component of type {}", typ);
        let key = format!("cdata:{}", typ);
        debug!("Serializing component");
        let time = Instant::now();
        let buffer = rkyv::api::high::to_bytes_with_alloc(&component, arena).map_err(|e| VKSError::RKYVError(RkyvError(Box::new(e))))?;
        debug!("Component serialized in {}ms", time.elapsed().as_millis());
        let len_key = format!("{}:len", key);
        let digest = self.index(&len_key).await?;
        trace!("Pushing component into DB");
        AsyncTypedCommands::hset(self, &key, digest, buffer.as_slice()).await?;
        Ok(format!("{}:{}", typ, digest))
    }

    async fn components_of_type(&mut self, typ: &str) -> Result<HashMap<String, Vec<u8>>, VKSError> {
        let key = format!("cdata:{}", typ);
        let map: HashMap<String, Vec<u8>> = AsyncCommands::hgetall(self, key).await?;
        Ok(map)
    }

    async fn get_entity(&mut self, id: isize) -> Result<Vec<String>, VKSError> {
        let key = format!("entities:{}", id);
        Ok(AsyncTypedCommands::lrange(self, key, 0, -1).await?)
    }

    async fn get_component<T, E>(&mut self, id: &str) -> Result<T, VKSError>
    where
        T: Archive,
        T::Archived: for<'a> CheckBytes<HighValidator<'a, E>>
            + Deserialize<T, Strategy<Pool, E>>,
        E: Source,
    {
        let split: Vec<&str> = id.split(":").collect();
        let typ = split[0];
        let id = split[1];
        let key = format!("cdata:{}", typ);
        let buffer: Vec<u8> = AsyncCommands::hget(self, key, id).await?;
        println!("buffer {:?}", buffer);
        Ok(rkyv::from_bytes(buffer.as_slice()).map_err(|e| VKSError::RKYVError(RkyvError(Box::new(e))))?)
    }

    async fn set_component<
        'a,
        E: Source
    >(
        &mut self,
        id: &str,
        component: &impl for<'b> Serialize<
            HighSerializer<AlignedVec, ArenaHandle<'b>, E>,
        >,
        arena: ArenaHandle<'a>
    ) -> Result<(), VKSError> {
        let split: Vec<_> = id.split(":").collect();
        let typ = split[0];
        let id = split[1];
        let typ_key = format!("cdata:{}", typ);
        let buffer = to_bytes_with_alloc(component, arena).map_err(|e| VKSError::RKYVError(RkyvError(Box::new(e))))?;
        AsyncTypedCommands::hset(self, &typ_key, id, buffer.as_slice()).await?;
        Ok(())
    }

    async fn set_entity(&mut self, id: isize, components: &[&str]) -> Result<(), VKSError> {
        let key = format!("entities:{}", id);
        AsyncTypedCommands::lrem(self, &key, 0, -1).await?;
        AsyncTypedCommands::lpush(self, &key, components).await?;
        Ok(())
    }
}

pub fn deserialize<'a, T, E>(data: &'a Vec<u8>) -> &'a T::Archived
where
    T: Archive + Send,
    T::Archived: for<'b> CheckBytes<HighValidator<'b, E>>
        + Deserialize<T, Strategy<Pool, E>>,
    E: Source,
{
    let pos = root_position::<T::Archived>(data.len());
    let archived = unsafe { access_pos_unchecked(data.as_slice(), pos) };
    archived
}

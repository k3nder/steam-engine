use std::collections::HashMap;

use rkyv::{
    Archive, Deserialize, Serialize,
    api::high::{HighSerializer, HighValidator},
    bytecheck::CheckBytes,
    de::Pool,
    rancor::{Source, Strategy},
    ser::allocator::ArenaHandle,
    util::AlignedVec,
};

pub trait AsyncPersistent {
    type Error: std::error::Error;
    /// function to obtain an index that grows for each call.
    /// ## Example
    /// ```rust
    /// let index = vks.index("counter")?;
    /// assert_eq!(index, 1);
    /// let index = vks.index("counter")?;
    /// assert_eq!(index, 2);
    /// ```
    fn index(&mut self, key: &str) -> impl Future<Output = Result<isize, Self::Error>>;
    /// function that generates a new entity with the provided components, returns the id of the entity
    /// ## Example
    /// ```rust
    /// let entity = vks.create_entity(&["render:13", "clock:45"])?;
    /// ```
    fn create_entity(
        &mut self,
        components: &[&str],
    ) -> impl Future<Output = Result<isize, Self::Error>>;
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
        component: impl for<'b> Serialize<HighSerializer<AlignedVec, ArenaHandle<'b>, E>> + Send,
        arena: ArenaHandle<'a>,
    ) -> impl Future<Output = Result<String, Self::Error>>;
    /// obtains all the components of a specific type
    /// ## Example
    /// ```rust
    /// let clocks = vks.components_of_type("clock")?;
    /// ```
    fn components_of_type(
        &mut self,
        typ: &str,
    ) -> impl Future<Output = Result<HashMap<String, Vec<u8>>, Self::Error>>;
    /// gets the components of an entity with its id
    /// ## Example
    /// ```rust
    /// let components = vks.get_entity("entities:1")?;
    /// ```
    fn get_entity(&mut self, id: isize) -> impl Future<Output = Result<Vec<String>, Self::Error>>;
    /// Gets the data of a specific component with its id
    /// ## Example
    /// ```rust
    /// let component = vks.get_component("clock:13");
    /// ```
    fn get_component<T, E>(&mut self, id: &str) -> impl Future<Output = Result<T, Self::Error>>
    where
        T: Archive,
        T::Archived: for<'a> CheckBytes<HighValidator<'a, E>> + Deserialize<T, Strategy<Pool, E>>,
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
    fn set_component<'a, E, C>(
        &mut self,
        id: &str,
        component: C,
        arena: ArenaHandle<'a>,
    ) -> impl Future<Output = Result<(), Self::Error>>
    where
        E: Source,
        C: for<'b> Serialize<HighSerializer<AlignedVec, ArenaHandle<'b>, E>> + Send;
    /// Replaces the data of an entity by other data
    /// ## Example
    /// ```rust
    /// vks.set_entity(2, &["clock:55"])
    /// ```
    fn set_entity(
        &mut self,
        id: isize,
        components: &[&str],
    ) -> impl Future<Output = Result<(), Self::Error>>;
    /// Sets a value to the persistent DB
    fn set(&mut self, id: &str, value: Vec<u8>) -> impl Future<Output = Result<(), Self::Error>>;
    /// Gets a value from the persistent DB
    fn get(&mut self, id: &str) -> impl Future<Output = Result<Vec<u8>, Self::Error>>;
}

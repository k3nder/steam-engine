## STEAMENGINE-VKS
Create part of steamengine, in charge of registering components and entities to a valkey server.

This crate converts each component to a binary structure with rkyv in record time, contains functions to obtain all components of a single type and allows the registration of new components as long as they can be serialized and deserialized with rkyv.

> [!NOTE]
> This crate is asynchronous and is intended exclusively for steamengine.

### EXAMPLE
```rust
#[derive(Debug, Deserialize, Serialize, Archive)]
struct PositionComponent {
    x: u32,
    y: u32,
    z: u32
}

let connection = VKSConnection::connect("redis://localhost").await?;
connection.create_component("position", PositionComponent {
    x: 43u32,
    y: 12u32,
    z: 234u32
}).await?;
```

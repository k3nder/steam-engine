## STEAMENGINE-PCS
crate part of steamengine, in charge of communication between client and server

This crate only implements utilities to TpcStream (including tokio with the ‘tokio-tcp’ feature) to read packets and write packets, and also has traits for that.

### EXAMPLE
```rust
pub struct TestPackage {
    pub message: String
}
impl ToPackage for TestPackage {
    pub fn to_package(self, alloc: ArenaHandle<'_>) -> Package {
        let value = to_bytes_with_alloc(self, alloc);
        Package {
            name: String::from("test"),
            value
        }
    }
}

let mut tpc = TcpStream::connect("localhost");
tcp.write_package(TestPackage {
    message: String::from("Hello")
})
```

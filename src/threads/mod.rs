/// This module is used to comunicate the threads
pub mod channel;

/// This macro is used to invokate a new thread
/// ## Example
/// ### Simple thread without communication
/// ```rust
/// let spawn = thread!({
///     loop {
///         println!("hello from thread!");
///     }
/// });
/// ```
///
/// ### Thread with comunication
/// ```rust
/// let comm = CommManager::new();
/// //      Thread ID, Communicator, Block
/// // Important! the id musn't be zero, zero is the main thread in the communicator
/// let spawn = thread!(1, comm, {
///     loop {
///         println!("Thread with communication");
///     }
/// });
/// ```
#[macro_export]
macro_rules! thread {
    ($body:block) => {{ std::thread::spawn(move || $body) }};
    ($id:expr, $communicator:expr, $body:expr) => {{
        if $id == 0 {
            panic!("Thread ID cannot be zero");
        }
        $communicator.register($id);
        let comm = $communicator.channel($id);
        std::thread::spawn(move || $body(comm))
    }};
    ($id:expr, $communicator:expr, $body:expr, $args:expr) => {{
        if $id == 0 {
            panic!("Thread ID cannot be zero");
        }
        $communicator.register($id);
        let comm = $communicator.channel($id);
        std::thread::spawn(move || $body(comm, $args))
    }};
}

pub mod channel;

#[macro_export]
macro_rules! thread {
    ($body:block) => {{ std::thread::spawn(move || $body) }};
    ($id:expr, $communicator:expr, $body:expr) => {{
        let comm = $communicator.channel($id);
        std::thread::spawn(move || $body(comm))
    }};
    ($id:expr, $communicator:expr, $body:expr, $args:expr) => {{
        let comm = $communicator.channel($id);
        std::thread::spawn(move || $body(comm, $args))
    }};
}

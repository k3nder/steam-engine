pub mod channel;

macro_rules! thread {
    ($id:expr, $communicator:ident, $body:ident) => {{
        let comm = $communicator.channel($id);
        std::thread::spawn(move || $body(comm))
    }};
    ($id:expr, $communicator:expr, $body:ident, $args:expr) => {{
        let comm = $communicator.channel($id);
        std::thread::spawn(move || $body(comm, $args))
    }};
    ($body:block) => {{ std::thread::spawn(move || $body) }};
}

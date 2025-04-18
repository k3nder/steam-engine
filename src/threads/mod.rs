pub mod channel;

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

#[macro_use]
pub mod threads;

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use std::{sync::Arc, thread, time::Duration};

    use crate::threads::channel::{self, Channel, CommManager, Message};

    use super::*;

    #[test]
    fn it_works() {
        let str = String::from("Hello, world!");
        let comm = channel::CommManager::from_threads(0..1);
        let thread = thread!(0, comm, test);

        for _ in 0..100 {
            comm.send_to(0, Message::Test);
        }
    }
    fn test(comm: Channel) {
        loop {
            println!("{:?}", comm.recv());
        }
    }
}

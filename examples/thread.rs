use steamengine::{
    thread,
    threads::channel::{Channel, CommManager, Event, Message},
};

fn main() {
    let mut manager = CommManager::new();
    let thread1 = thread!(1, manager, thread, 1);
    let thread2 = thread!(2, manager, thread, 2);
    let thread3 = thread!(3, manager, thread, 3);

    manager.broadcast(Message::Test).unwrap();
    manager
        .send_to(1, Message::String("Hello number 1".to_owned()))
        .unwrap();
    manager.send_to(2, Message::Bool(true)).unwrap();
    manager
        .send_to(3, Message::String("Hello number 3".to_owned()))
        .unwrap();

    manager.broadcast(Message::Event(Event::Exit)).unwrap();

    thread1.join().unwrap();
    thread2.join().unwrap();
    thread3.join().unwrap();
}

fn thread(channel: Channel, number: u32) {
    loop {
        let rec = channel.recv().unwrap();
        match rec {
            Message::Event(Event::Exit) => break,
            _ => {}
        }
        println!("{} Receiving signal: {:?}", number, rec);
    }
}

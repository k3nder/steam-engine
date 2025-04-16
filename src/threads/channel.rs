use std::{collections::HashMap, ops::Range, sync::Arc};

use crossbeam_channel::{Receiver, RecvError, Sender, TryRecvError};

/// Manages communication between threads.
pub struct CommManager {
    recivers: HashMap<usize, Arc<Receiver<Message>>>,
    senders: HashMap<usize, Arc<Sender<Message>>>,
}
/// handles the communication, from a specific thread
pub struct Channel {
    senders: HashMap<usize, Arc<Sender<Message>>>,
    reciver: Arc<Receiver<Message>>,
}

impl Channel {
    /// Creates a new channel with the given senders and receiver.
    pub(crate) fn new(
        senders: HashMap<usize, Arc<Sender<Message>>>,
        reciver: Arc<Receiver<Message>>,
    ) -> Self {
        Channel { senders, reciver }
    }

    /// Sends a message to the specified thread.
    pub fn send(&self, id: usize, message: Message) {
        if let Some(sender) = self.senders.get(&id) {
            sender.send(message).expect("error sending message");
        }
    }
    /// Receives a message from the channel(wait until you receive something).
    pub fn recv(&self) -> Result<Message, RecvError> {
        self.reciver.recv()
    }

    /// Receives a message from the channel.
    pub fn try_recv(&self) -> Result<Message, TryRecvError> {
        self.reciver.try_recv()
    }

    /// Sends a message to all registered threads.
    pub fn broadcast(&self, message: Message) {
        for (_, sender) in self.senders.iter() {
            sender.send(message.clone()).expect("error sending message");
        }
    }
}

impl CommManager {
    /// Creates a new communication manager with no registered threads.
    pub fn new() -> Self {
        CommManager {
            senders: HashMap::new(),
            recivers: HashMap::new(),
        }
    }

    /// Creates a new communication manager with the specified threads.
    pub fn from_threads(threads: Range<usize>) -> Self {
        let mut communicator = CommManager::new();
        for index in threads {
            communicator.register(index);
        }
        communicator
    }

    /// Registers a new thread with the communication manager.
    pub fn register(&mut self, id: usize) {
        let (sender, receiver) = crossbeam_channel::unbounded();
        self.recivers.insert(id, Arc::new(receiver));
        self.senders.insert(id, Arc::new(sender));
    }

    /// Sends a message to the specified thread.
    pub fn send_to(&self, id: usize, message: Message) {
        if let Some(sender) = self.senders.get(&id) {
            sender.send(message).expect("error sending message");
        }
    }

    /// Sends a message to all registered threads.
    pub fn broadcast(&self, message: Message) {
        for (_, sender) in self.senders.iter() {
            sender.send(message.clone()).expect("error sending message");
        }
    }

    /// Creates a new channel for communication with the specified thread.
    pub fn channel(&self, id: usize) -> Channel {
        Channel::new(
            self.senders.clone(),
            self.recivers.get(&id).unwrap().clone(),
        )
    }
}
/// Messages you can send to threads
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Message {
    // TODO: Define your message variants here
    Test,
}

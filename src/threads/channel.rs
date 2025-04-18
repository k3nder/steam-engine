use std::{collections::HashMap, ops::Range, sync::Arc};

use crossbeam_channel::{Receiver, RecvError, SendError, Sender, TryRecvError};
use log::error;

/// Manages communication between threads.
pub struct CommManager {
    receivers: HashMap<usize, Arc<Receiver<Message>>>,
    senders: HashMap<usize, Arc<Sender<Message>>>,
}
/// handles the communication, from a specific thread
pub struct Channel {
    senders: HashMap<usize, Arc<Sender<Message>>>,
    pub receiver: Arc<Receiver<Message>>,
}

impl Channel {
    /// Creates a new channel with the given senders and receiver.
    pub(crate) fn new(
        senders: HashMap<usize, Arc<Sender<Message>>>,
        receiver: Arc<Receiver<Message>>,
    ) -> Self {
        Channel { senders, receiver }
    }

    /// Sends a message to the specified thread.
    pub fn send(&self, id: usize, message: Message) -> Result<(), SendError<Message>> {
        if let Some(sender) = self.senders.get(&id) {
            return sender.send(message);
        }
        Ok(())
    }
    /// Receives a message from the channel(wait until you receive something).
    pub fn recv(&self) -> Result<Message, RecvError> {
        self.receiver.recv()
    }

    /// Receives a message from the channel.
    pub fn try_recv(&self) -> Result<Message, TryRecvError> {
        self.receiver.try_recv()
    }

    /// Sends a message to all registered threads.
    pub fn broadcast(&self, message: Message) -> Result<(), SendError<Message>> {
        for (_, sender) in self.senders.iter() {
            match sender.send(message.clone()) {
                Ok(_) => (),
                Err(err) => return Err(err),
            }
        }
        Ok(())
    }
}

impl CommManager {
    /// Creates a new communication manager with no registered threads.
    pub fn new() -> Self {
        let mut communicator = CommManager {
            senders: HashMap::new(),
            receivers: HashMap::new(),
        };
        communicator.register(0);
        communicator
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
        if self.receivers.contains_key(&id) {
            error!("Thread already registered");
        }
        let (sender, receiver) = crossbeam_channel::unbounded();
        self.receivers.insert(id, Arc::new(receiver));
        self.senders.insert(id, Arc::new(sender));
    }

    /// Sends a message to the specified thread.
    pub fn send_to(&self, id: usize, message: Message) -> Result<(), SendError<Message>> {
        if let Some(sender) = self.senders.get(&id) {
            return sender.send(message);
        }
        Ok(())
    }

    /// Sends a message to all registered threads.
    pub fn broadcast(&self, message: Message) -> Result<(), SendError<Message>> {
        for (_, sender) in self.senders.iter() {
            match sender.send(message.clone()) {
                Ok(_) => (),
                Err(err) => return Err(err),
            }
        }
        Ok(())
    }

    /// Creates a new channel for communication with the specified thread.
    pub fn channel(&self, id: usize) -> Channel {
        Channel::new(
            self.senders.clone(),
            self.receivers.get(&id).unwrap().clone(),
        )
    }

    /// Drops the specified communication channel.
    pub fn drop(&mut self, id: usize) {
        self.senders.remove(&id);
        self.receivers.remove(&id);
    }

    /// Recives a message reading the first(0) reciver
    /// *BLOCKING*
    pub fn recv(&self) -> Result<Message, RecvError> {
        self.receivers.get(&0).unwrap().recv()
    }

    /// Recives a message reading the first(0) reciver
    /// *MON BLOCKING*
    pub fn try_recv(&self) -> Result<Message, TryRecvError> {
        self.receivers.get(&0).unwrap().try_recv()
    }
}
/// Messages you can send to threads
#[derive(Debug, Clone, PartialEq)]
pub enum Message {
    Test,
    String(String),
    Int(i32),
    Float(f32),
    Bool(bool),
    Char(char),
    Event(Event),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Event {
    Start,
    Stop,
    Pause,
    Resume,
    Exit,
}

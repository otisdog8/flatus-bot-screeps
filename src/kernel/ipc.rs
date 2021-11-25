use std::collections::{HashMap, VecDeque};

use bytecheck::CheckBytes;
use rkyv::{with::AsVec, Archive, Deserialize, Serialize};
use screeps::Part;

// IPC is

#[derive(Archive, Serialize, Deserialize)]
#[archive_attr(derive(CheckBytes))]
pub enum InterProcessCommunication {
    A(InterProcessCommunicationA),
    B(InterProcessCommunicationB),
}

// When rkyv supports VecDeque upstream, update datastruct
#[derive(Archive, Serialize, Deserialize)]
#[archive_attr(derive(CheckBytes))]
pub struct InterProcessCommunicationA {
    messages: HashMap<u32, Vec<Message>>,
    channels: HashMap<u32, Vec<Message>>,
}

impl InterProcessCommunicationA {
    pub fn new() -> InterProcessCommunicationA {
        InterProcessCommunicationA {
            messages: HashMap::new(),
            channels: HashMap::new(),
        }
    }

    pub fn send_message_pid(&mut self, pid: u32, message: Message) {
        match self.messages.get_mut(&pid) {
            Some(a) => {
                a.push(message);
            }
            None => {
                let mut message_queue = Vec::new();
                message_queue.push(message);
                self.messages.insert(pid, message_queue);
            }
        }
    }

    pub fn send_message_channel(&mut self, channel: u32, message: Message) {
        match self.messages.get_mut(&channel) {
            Some(a) => {
                a.push(message);
            }
            None => {
                let mut channel_queue = Vec::new();
                channel_queue.push(message);
                self.channels.insert(channel, channel_queue);
            }
        }
    }

    //pub fn get_message_pid(&mut self, pid: u32) -> Option<Message> {
    //    match self.messages.get_mut(&pid) {
    //        Some(a) => a.pop_front(),
    //        None => None,
    //    }
    //}

    pub fn get_messages_pid(&mut self, pid: u32) -> Option<&mut Vec<Message>> {
        self.messages.get_mut(&pid)
    }

    pub fn rm_pid(&mut self, pid: u32) {
        self.messages.remove(&pid);
    }

    //pub fn get_message_channel(&mut self, channel: u32) -> Option<Message> {
    //    match self.channels.get_mut(&channel) {
    //        Some(a) => a.pop_front(),
    //        None => None,
    //    }
    //}

    pub fn get_messages_channel(&mut self, channel: u32) -> Option<&mut Vec<Message>> {
        self.channels.get_mut(&channel)
    }

    pub fn rm_channel(&mut self, channel: u32) {
        self.channels.remove(&channel);
    }
}
#[derive(Archive, Serialize, Deserialize)]
#[archive_attr(derive(CheckBytes))]
pub struct InterProcessCommunicationB {}

impl InterProcessCommunicationB {
    pub fn new() -> InterProcessCommunicationB {
        InterProcessCommunicationB {}
    }
}

pub fn new_ipc_as_enum() -> InterProcessCommunication {
    InterProcessCommunication::A(InterProcessCommunicationA::new())
}

pub fn get_ipc(borrow: &InterProcessCommunication) -> &InterProcessCommunicationA {
    match borrow {
        InterProcessCommunication::A(a) => a,
        InterProcessCommunication::B(_) => panic!("DN"),
    }
}

pub fn get_mut_ipc(borrow: &mut InterProcessCommunication) -> &mut InterProcessCommunicationA {
    match borrow {
        InterProcessCommunication::A(a) => a,
        InterProcessCommunication::B(_) => panic!("DN"),
    }
}

// Message is what people will pass around
// Specify representations so they don't change and mess with stuff
#[derive(Archive, Serialize, Deserialize)]
#[archive_attr(derive(CheckBytes))]
#[archive_attr(repr(u16))]
#[repr(u16)]
pub enum Message {
    SpawnRequest(SpawnRequest) = 1,
    SpawnResponse(SpawnResponse) = 2,
    AssetDonation(AssetDonation) = 3,
}

#[derive(Archive, Serialize, Deserialize)]
#[archive_attr(derive(CheckBytes))]
pub struct SpawnRequest {
    origin: u32,
    // Need to parse parts back
    parts: Vec<u8>,
    name: String,
    largest_available: bool,
    dir: u8,
}

// Empty if failed -
#[derive(Archive, Serialize, Deserialize)]
#[archive_attr(derive(CheckBytes))]
pub struct SpawnResponse {
    pub name: String,
}

// NOT IMPLEMENTED YET
#[derive(Archive, Serialize, Deserialize)]
#[archive_attr(derive(CheckBytes))]
pub struct AssetDonation {
    pub name: String,
    pub asset_type: u8,
    pub location: u8,
}

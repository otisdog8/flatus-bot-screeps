use crate::{
    ipc::InterProcessCommunicationA, kernel::KernelA, refcell_serialization::InlineRefCell,
};
use bytecheck::CheckBytes;
use rkyv::{
    archived_value,
    ser::{serializers::AllocSerializer, Serializer},
    Archive, Archived, Deserialize, Infallible, Serialize,
};
use std::cell::RefCell;

#[derive(Archive, Serialize, Deserialize)]
#[archive_attr(derive(CheckBytes))]
#[archive_attr(repr(u16))]
#[impl_enum::with_methods {
    pub fn can_run(&self) -> bool {}
    pub fn get_pid(&self) -> u32 {}
    pub fn set_pid(&mut self, pid: u32) {}
    pub fn get_prio(&self) -> u8 {}
    pub fn set_prio(&mut self, prio: u8) {}
    pub fn get_ptype(&self) -> u16 {}
    pub fn kill(&mut self) {}
    pub fn get_child_processes(&self) -> Vec<u32> {}
    pub fn run(&mut self, ipc: &mut InterProcessCommunicationA, kernel: &KernelA, cache: u32) -> u16 {}
}]
#[repr(u16)]
pub enum Process {
    TestProcessA(TestProcessA) = 1,
    TestProcessB(TestProcessA) = 2,
}

#[derive(Archive, Serialize, Deserialize)]
#[archive_attr(derive(CheckBytes))]
pub struct ProcessContainer {
    #[with(InlineRefCell)]
    pub val: RefCell<Process>,
}

impl ProcessContainer {
    pub fn new() -> ProcessContainer {
        ProcessContainer {
            val: RefCell::new(Process::TestProcessA(TestProcessA::new())),
        }
    }
}

// Current version of TestProcess is A

#[derive(Archive, Serialize, Deserialize)]
#[archive_attr(derive(CheckBytes))]
pub struct TestProcessA {
    pid: u32,
}

impl TestProcessA {
    fn can_run(&self) -> bool {
        true
    }

    fn get_pid(&self) -> u32 {
        0
    }

    fn set_pid(&mut self, pid: u32) {
        self.pid = pid;
    }

    fn get_prio(&self) -> u8 {
        0
    }

    fn set_prio(&mut self, prio: u8) {}

    fn get_ptype(&self) -> u16 {
        0
    }

    fn kill(&self) {
        // Recursively kill child processes
    }

    fn spawn_child_process(&self) {}

    fn get_child_processes(&self) -> Vec<u32> {
        Vec::new()
    }

    fn run(&mut self, ipc: &mut InterProcessCommunicationA, kernel: &KernelA, cache: u32) -> u16 {
        0
    }

    fn new() -> TestProcessA {
        TestProcessA { pid: 0 }
    }
}

// Can_run
// Run_process
// Think about how to handle spawns
// Per-room spawn queues
//

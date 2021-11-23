use rkyv::{
    archived_value,
    ser::{serializers::AllocSerializer, Serializer},
    Archive, Archived, Deserialize, Infallible, Serialize,
};
use rkyv_dyn::archive_dyn;
use rkyv_typename::TypeName;

#[archive_dyn(deserialize)]
pub trait Process {
    fn get_priority(&self) -> u8;

    // Blocks on tick or interprocess communication
    fn can_run(&self) -> bool;

    fn get_pid(&self) -> u16;

    fn get_ptype(&self) -> u16;

    fn term(&self) -> bool;

    fn kill(&self);

    fn spawn_child_process(&self);

    fn get_child_processes(&self) -> Vec<u16>;

    fn run(&mut self) -> u16;
}

#[derive(Archive, Serialize, Deserialize)]
#[archive_attr(derive(TypeName))]
struct TestProcess {}

#[archive_dyn(deserialize)]
impl Process for TestProcess {
    fn get_priority(&self) -> u8 {
        0
    }

    fn can_run(&self) -> bool {
        true
    }

    fn get_pid(&self) -> u16 {
        0
    }

    fn get_ptype(&self) -> u16 {
        0
    }

    fn term(&self) -> bool {
        true
    }

    fn kill(&self) {}

    fn spawn_child_process(&self) {}

    fn get_child_processes(&self) -> Vec<u16> {
        Vec::new()
    }

    fn run(&mut self) -> u16 {
        0
    }
}

impl Process for Archived<TestProcess> {
    fn get_priority(&self) -> u8 {
        0
    }

    fn can_run(&self) -> bool {
        true
    }

    fn get_pid(&self) -> u16 {
        0
    }

    fn get_ptype(&self) -> u16 {
        0
    }

    fn term(&self) -> bool {
        true
    }

    fn kill(&self) {}

    fn spawn_child_process(&self) {}

    fn get_child_processes(&self) -> Vec<u16> {
        Vec::new()
    }

    fn run(&mut self) -> u16 {
        0
    }
}


// Can_run
// Run_process
// Think about how to handle spawns
// Per-room spawn queues
//
use bytecheck::CheckBytes;
use log::*;
use rkyv::check_archived_root;
use rkyv::{
    archived_root,
    ser::{serializers::AllocSerializer, Serializer},
    Archive, Deserialize, Infallible, Serialize,
};
use screeps::RawMemory;
use std::{borrow::Borrow, cell::RefCell, rc::Rc};

use crate::performance::new_perflog_as_enum;
use crate::{
    performance::{self, PerfLog},
    refcell_serialization, strlib,
};

thread_local! {
    pub static MEMORY: RefCell<Memory> = RefCell::new(Memory::A(MemoryA::new()));
}

#[derive(Archive, Serialize, Deserialize)]
#[archive_attr(derive(CheckBytes))]
pub enum Memory {
    A(MemoryA),
    B(MemoryB),
}

#[derive(Archive, Serialize, Deserialize)]
#[archive_attr(derive(CheckBytes))]
pub struct MemoryA {
    pub test: u32,
    #[with(refcell_serialization::InlineRefCell)]
    pub test2: RefCell<u32>,
    #[with(refcell_serialization::InlineRefCell)]
    pub perf: RefCell<performance::PerfLog>,
}
// Implement entirely new instances in the event of a cold boot
impl MemoryA {
    pub fn new() -> MemoryA {
        MemoryA {
            test: 0,
            test2: RefCell::new(0),
            perf: RefCell::new(performance::new_perflog_as_enum()),
        }
    }
}

#[derive(Archive, Serialize, Deserialize)]
#[archive_attr(derive(CheckBytes))]
pub struct MemoryB {
    pub test: u8,
    #[with(refcell_serialization::InlineRefCell)]
    pub test2: RefCell<u16>,
}
// Implement entirely new instances in the event of a cold boot
impl MemoryB {
    pub fn new() -> MemoryB {
        MemoryB {
            test: 0,
            test2: RefCell::new(0),
        }
    }
}

// PLEASE REMEMBER TO DISABLE MIGRATIONS AFTER THEY HAVE OCCURRED
// CURRENT MEMORY VERSION IS A
const MIGRATE: bool = false;

pub fn new_memory_as_enum() -> Memory {
    // Cold boot Memory generator
    Memory::A(MemoryA::new())
}

// Function to return current memory version
// Requires borrow as the instance of Memory, will put memory_var in scope as <current memory version>
pub fn get_memory(borrow: &Memory) -> &MemoryA {
    match borrow {
        Memory::A(a) => {
            // Memory should never be a MemoryA
            a
        }
        Memory::B(b) => panic!("DN"),
    }
}

// -1 for cold boot, 0 for normal
pub fn init() -> u8 {
    let memory = RawMemory::get();
    if memory.length() == 0 {
        // This only happens if Memory got wiped somehow (bad)
        MEMORY.with(|memory_refcell| {
            memory_refcell.replace(new_memory_as_enum());
        });
        return 255;
    } else {
        let memory_bytestring = strlib::jsstring_to_bytestring(&memory);
        let zcp_memory = check_archived_root::<Memory>(&memory_bytestring).unwrap();
        let deserialized_memory: Memory = zcp_memory.deserialize(&mut Infallible).unwrap();
        // Code to migrate from one version of the Memory struct to the next
        // B -> A current migration
        match deserialized_memory {
            Memory::A(_) => {
                MEMORY.with(|memory_refcell| {
                    memory_refcell.replace(deserialized_memory);
                });
            }
            Memory::B(old_memory) => {
                let old_test = old_memory.test;
                let new_test = old_test as u32;
                let old_test2 = old_memory.test2.into_inner() as u32;
                let new_test2 = RefCell::new(old_test2);
                let perf = RefCell::new(new_perflog_as_enum());
                MEMORY.with(|memory_refcell| {
                    memory_refcell.replace(Memory::A(MemoryA {
                        test: new_test,
                        test2: new_test2,
                        perf: perf,
                    }));
                });
            }
        }
    }
    // Deserialize Memory from JsString to Vec<u8>
    // Copy Deserialize Vec<u8> to Memory enum
    // If type != CURRENT_MEMORY_VERSION, migrate (code this manually) and change Structs
    // If there is a failure somewhere along here, it means we probably don't have Memory/have incoherent Memory.
    // In that case, we "cold boot" regenerate memory.
    // Also make sure to place the value in MEMORY
    // Also make sure to log performance somehow
    0
}

pub fn save() {
    let mut serializer = AllocSerializer::<256>::default();
    MEMORY.with(|memory_refcell| {
        let borrow = &*memory_refcell.borrow();
        match serializer.serialize_value(borrow) {
            Ok(_) => (),
            Err(_) => (),
        };
    });
    let bytes = serializer.into_serializer().into_inner();
    let bytes = bytes.into_boxed_slice();
    let js_string = strlib::bytestring_to_jsstring(&bytes);
    RawMemory::set(&js_string);

    // Serialize Memory from Enum to Vec<u8>
    // Serialize Vec<u8> into JsString
    // Make sure to log performance
}

use bytecheck::CheckBytes;
use rkyv::{
    archived_root,
    ser::{serializers::AllocSerializer, Serializer},
    Archive, Deserialize, Infallible, Serialize,
};
use rkyv::check_archived_root;
use screeps::RawMemory;
use std::{borrow::Borrow, cell::RefCell, rc::Rc};

use crate::strlib;

thread_local! {
    static MEMORY: RefCell<Memory> = RefCell::new(Memory::A(MemoryA { test: 255 }));
}

#[derive(Archive, Serialize, Deserialize)]
#[archive_attr(derive(CheckBytes))]
enum Memory {
    A(MemoryA),
    B(MemoryB),
}

#[derive(Archive, Serialize, Deserialize)]
#[archive_attr(derive(CheckBytes))]
struct MemoryA {
    test: u32,
}

#[derive(Archive, Serialize, Deserialize)]
#[archive_attr(derive(CheckBytes))]
struct MemoryB {
    test: u8,
}

// PLEASE REMEMBER TO DISABLE MIGRATIONS AFTER THEY HAVE OCCURRED
// CURRENT MEMORY VERSION IS A
const MIGRATE: bool = true;

// -1 for cold boot, 0 for normal
pub fn init() -> u8 {
    let memory = RawMemory::get();
    let memory_bytestring = strlib::jsstring_to_bytestring(&memory);
    let zcp_memory = check_archived_root::<Memory>(&memory_bytestring).unwrap();
    let deserialized_memory: Memory = zcp_memory.deserialize(&mut Infallible).unwrap();
    if !MIGRATE {
        MEMORY.with(|memory_refcell| {
            memory_refcell.replace(deserialized_memory);
        });
    }
    else {
        // Code to migrate from one version of the Memory struct to the next
        // A -> B current migration
        match deserialized_memory {
            Memory::A(old_memory) => {
                let old_test = old_memory.test;
                let new_test = old_test as u8;
                MEMORY.with(|memory_refcell| {
                    memory_refcell.replace(Memory::B(MemoryB { test: new_test }));
                });
            }
            Memory::B(old_memory) => {
                let old_test = old_memory.test;
                let new_test = old_test as u32;
                MEMORY.with(|memory_refcell| {
                    memory_refcell.replace(Memory::A(MemoryA { test: new_test }));
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

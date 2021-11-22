use bytecheck::CheckBytes;
use rkyv::{
    archived_root,
    ser::{serializers::AllocSerializer, Serializer},
    Archive, Deserialize, Infallible, Serialize,
};
use rkyv::check_archived_root;
use screeps::RawMemory;
use std::{borrow::Borrow, cell::RefCell, rc::Rc};

use crate::{strlib, refcell_serialization};

thread_local! {
    pub static MEMORY: RefCell<Memory> = RefCell::new(Memory::A(MemoryA { test: 255 }));
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
}

#[derive(Archive, Serialize, Deserialize)]
#[archive_attr(derive(CheckBytes))]
pub struct MemoryB {
    pub test: u8,
    #[with(refcell_serialization::InlineRefCell)]
    pub test2: RefCell<u16>,
}

// PLEASE REMEMBER TO DISABLE MIGRATIONS AFTER THEY HAVE OCCURRED
// CURRENT MEMORY VERSION IS B
const MIGRATE: bool = false;

// Macro to automatically run with the current memory version
// Requires borrow as the instance of Memory, will put memory_var in scope as MemoryB
#[macro_export]
macro_rules! mem {
    ( $borrow:ident, $code: block) => {
        {
            match $borrow {
                crate::memory::Memory::A(memory_var) => (),
                crate::memory::Memory::B(memory_var) => {
                    $code;
                }
            }
        }
    };
}

#[macro_export]
macro_rules! mem2 {
    ( $borrow:ident) => {
        {
            match $borrow {
                crate::memory::Memory::A(memory_var) => (),
                crate::memory::Memory::B(memory_var) => {
                    memory_var
                }
            }
        }
    };
}


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
                    memory_refcell.replace(Memory::B(MemoryB { test: new_test, test2: RefCell::new(new_test as u16) }));
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

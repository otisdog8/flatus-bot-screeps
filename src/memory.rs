use core::slice::SlicePattern;

use js_sys::JsString;
use rkyv::ser::serializers::AllocSerializer;
use screeps::RawMemory;

use crate::strlib;
// Currently just dropping u8s - 50% storage efficiency loss
// Sec

thread_local! {
    static MEMORY: RefCell<Memory> = RefCell::new(Memory::A(MemoryA { test: 16 }));
}

#[derive(Archive, Serialize, Deserialize)]
enum Memory {
    A(MemoryA),
    B(MemoryB),
}

#[derive(Archive, Serialize, Deserialize)]
struct MemoryA {
    test: u32,
}

#[derive(Archive, Serialize, Deserialize)]
struct MemoryB {
    test: u8,
}

// Memory does get some functions that are implemented, but these need to change depending on the version
const type CURRENT_MEMORY_VERSION = Memory::A();

impl MemoryA {
}

fn init() {
    let memory = RawMemory::get();
    let memory_bytestring = strlib::jsstring_to_bytestring(&memory);
    let test = CURRENT_MEMORY_VERSION();

    // Deserialize Memory from JsString to Vec<u8>
    // Copy Deserialize Vec<u8> to Memory enum
    // If type != CURRENT_MEMORY_VERSION, migrate (code this manually) and change Structs
    // If there is a failure somewhere along here, it means we probably don't have Memory/have incoherent Memory.
    // In that case, we "cold boot" regenerate memory.
    // Also make sure to place the value in MEMORY
    // Also make sure to log performance somehow
}

fn save() {
    let mut serializer = AllocSerializer::<256>::default();
    // Serialize Memory from Enum to Vec<u8>
    // Serialize Vec<u8> into JsString
    // Make sure to log performance
}



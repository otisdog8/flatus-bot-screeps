#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]
#![allow(unused_imports)]
#![allow(unused_macros)]
use std::cell::RefCell;
use std::collections::HashMap;

use log::*;
use screeps::{
    find, game, prelude::*, Creep, ObjectId, Part, ResourceType, ReturnCode, RoomObjectProperties,
    Source, StructureController, StructureObject,
};
use wasm_bindgen::prelude::*;

mod logging;
mod memory;
mod strlib;
mod refcell_serialization;

// add wasm_bindgen to any function you would like to expose for call from js
#[wasm_bindgen]
pub fn setup() {
    logging::setup_logging(logging::Info);
    memory::init();
}

// to use a reserved name as a function name, use `js_name`:
#[wasm_bindgen(js_name = loop)]
pub fn game_loop() {
    memory::MEMORY.with(|memory_refcell| {
        let borrow = &*memory_refcell.borrow();
        match borrow {
            memory::Memory::A(a) => (),
            memory::Memory::B(b) => {

                info!("aaaa");
                let val = b.test2.borrow();
                info!("Initial value of memory location {}", &*val);
                let newval = &*val+1;
                drop(val);
                let a = b.test2.replace(newval);
                info!("Final value of memory location {}", b.test2.borrow());
            }
        }
    });
    // Game::spawns returns a `js_sys::Object`, which is a light reference to an
    // object of any kind which is held on the javascript heap.
    //
    // Object::values returns a `js_sys::Array`, which contains the member spawn objects
    // representing all the spawns you control.
    //
    // They are returned as wasm_bindgen::JsValue references, which we can safely
    // assume are StructureSpawn objects as returned from js without checking first
    memory::save();
    info!("done! cpu: {}", game::cpu::get_used())
}


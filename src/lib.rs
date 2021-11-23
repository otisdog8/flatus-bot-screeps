#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]
#![allow(unused_imports)]
#![allow(unused_macros)]
#![allow(unused_unsafe)]

use std::cell::RefCell;
use std::collections::HashMap;

use log::*;
use performance::{get_mut_perflog, get_perflog, get_taskdata, get_tasklog};
use screeps::{
    find, game, prelude::*, Creep, ObjectId, Part, ResourceType, ReturnCode, RoomObjectProperties,
    Source, StructureController, StructureObject, StructureSpawn,
};
use wasm_bindgen::prelude::*;

mod logging;
mod memory;
mod performance;
mod refcell_serialization;
mod strlib;
mod process;
mod process_table;
mod game_cache;

use crate::memory::{get_memory, Memory, MEMORY};

// add wasm_bindgen to any function you would like to expose for call from js
#[wasm_bindgen]
pub fn setup() {
    logging::setup_logging(logging::Info);
    memory::init();
}

// to use a reserved name as a function name, use `js_name`:
#[wasm_bindgen(js_name = loop)]
pub fn game_loop() {
    // Any pretick stuff independent of the OS should run here

    // Give perf the tick so it can cache it
    let tick = screeps::game::time();
    MEMORY.with(|memory_refcell| {
        let borrow = &*memory_refcell.borrow();
        let memory_var = get_memory(borrow);
        let perf = &mut *memory_var.perf.borrow_mut();
        let perf = get_mut_perflog(perf);
        perf.tick = tick;
    });
    perf!(1,
        memory::MEMORY.with(|memory_refcell| {
            let borrow = &*memory_refcell.borrow();
            let memory_var = get_memory(borrow);
            let val = memory_var.test2.borrow();
            let newval = &*val + 1;
            drop(val);
            let a = memory_var.test2.replace(newval);
        });
    );
    MEMORY.with(|memory_refcell| {
        let borrow = &*memory_refcell.borrow();
        let memory_var = get_memory(borrow);
        let perf = &*memory_var.perf.borrow();
        let perf = get_perflog(perf);
        for (key, value) in &perf.perf_data {
            let data = get_taskdata(value);
            info!("k {} v {}", key, data.get(false));
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

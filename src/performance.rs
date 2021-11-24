use std::{cell::RefCell, collections::HashMap};

use crate::memory::{get_memory, Memory, MEMORY};
use bytecheck::CheckBytes;
use rkyv::{Archive, Deserialize, Serialize};

// Here is where I log performance by averaging CPU deltas for processes
// Keep n deltas and move to a new delta every m, evict oldest deltas every once and a while too
// Index by hashmap and primitive strings - keep 2 deltas of 200 each (so a tuple)
// Keep this in Memory to aid scheduling, but offload to fs if bucket permitting (make fs offload a process with low priority)
// No 2 processes should be accessing this at the same time, which means we can keep the entire thing behind a RefCell

#[derive(Archive, Serialize, Deserialize)]
#[archive_attr(derive(CheckBytes))]
pub enum PerfLog {
    A(PerfLogA),
    B(PerfLogB),
}
#[derive(Archive, Serialize, Deserialize)]
#[archive_attr(derive(CheckBytes))]
pub struct PerfLogA {
    pub perf_data: HashMap<u16, TaskData>,
    pub current: HashMap<u16, TaskLog>,
    // Serialize tick so we only need to ask JS for the tick once
    pub tick: u32,
}

impl PerfLogA {
    pub fn new() -> PerfLogA {
        PerfLogA {
            perf_data: HashMap::new(),
            current: HashMap::new(),
            tick: 0,
        }
    }

    pub fn add_val(&mut self, key: u16, val: f64) {
        // Add individual task data
        match self.perf_data.get_mut(&key) {
            Some(data) => {
                let data = get_mut_taskdata(data);
                data.add(val);
            }
            None => {
                let mut data = new_taskdata_as_enum();
                let taskdata = get_mut_taskdata(&mut data);
                taskdata.add(val);
                self.perf_data.insert(key, data);
            }
        }
        // Add data for grafana
        match self.current.get_mut(&key) {
            Some(data) => {
                let data = get_mut_tasklog(data);
                data.add(self.tick, val);
            }
            None => {
                let mut data = new_tasklog_as_enum();
                let tasklog = get_mut_tasklog(&mut data);
                tasklog.add(self.tick, val);
                self.current.insert(key, data);
            }
        }
    }
}

#[derive(Archive, Serialize, Deserialize)]
#[archive_attr(derive(CheckBytes))]
pub struct PerfLogB {}

impl PerfLogB {
    pub fn new() -> PerfLogB {
        PerfLogB {}
    }
}

pub fn new_perflog_as_enum() -> PerfLog {
    PerfLog::A(PerfLogA::new())
}

pub fn get_perflog(borrow: &PerfLog) -> &PerfLogA {
    match borrow {
        PerfLog::A(a) => a,
        PerfLog::B(_) => panic!("DN"),
    }
}

pub fn get_mut_perflog(borrow: &mut PerfLog) -> &mut PerfLogA {
    match borrow {
        PerfLog::A(a) => a,
        PerfLog::B(_) => panic!("DN"),
    }
}

#[derive(Archive, Serialize, Deserialize)]
#[archive_attr(derive(CheckBytes))]
pub enum TaskData {
    A(TaskDataA),
    B(TaskDataB),
}
#[derive(Archive, Serialize, Deserialize)]
#[archive_attr(derive(CheckBytes))]
pub struct TaskDataA {
    pub range1: (u8, f64),
    pub range2: (u8, f64),
    pub total: u64,
}

impl TaskDataA {
    pub fn new() -> TaskDataA {
        TaskDataA {
            range1: (0, 0f64),
            range2: (128, 0f64),
            total: 0,
        }
    }

    pub fn add(&mut self, num: f64) {
        self.range1.1 = self.range1.0 as f64 * self.range1.1 + num;
        self.range1.0 = self.range1.0.wrapping_add(1);
        if self.range1.0 == 0 {
            self.range1.1 /= 256 as f64
        } else {
            self.range1.1 /= self.range1.0 as f64
        }
        self.range2.1 = self.range2.0 as f64 * self.range2.1 + num;
        self.range2.0 = self.range2.0.wrapping_add(1);
        if self.range2.0 == 0 {
            self.range2.1 /= 256 as f64
        } else {
            self.range2.1 /= self.range2.0 as f64
        }
        self.total += 1;
    }

    pub fn get(&self, largest: bool) -> f64 {
        // gets the largest or smallest - largest is most reliable, smallest is most recent
        if self.total > 127 {
            if !((self.range1.0 > self.range1.0) ^ largest) {
                self.range1.1
            } else {
                self.range2.1
            }
        } else {
            self.range1.1
        }
    }
}
#[derive(Archive, Serialize, Deserialize)]
#[archive_attr(derive(CheckBytes))]
pub struct TaskDataB {}

impl TaskDataB {
    pub fn new() -> TaskDataB {
        TaskDataB {}
    }
}

pub fn new_taskdata_as_enum() -> TaskData {
    TaskData::A(TaskDataA::new())
}

pub fn get_taskdata(borrow: &TaskData) -> &TaskDataA {
    match borrow {
        TaskData::A(a) => a,
        TaskData::B(_) => panic!("DN"),
    }
}

pub fn get_mut_taskdata(borrow: &mut TaskData) -> &mut TaskDataA {
    match borrow {
        TaskData::A(a) => a,
        TaskData::B(_) => panic!("DN"),
    }
}

#[derive(Archive, Serialize, Deserialize)]
#[archive_attr(derive(CheckBytes))]
pub enum TaskLog {
    A(TaskLogA),
    B(TaskLogB),
}
#[derive(Archive, Serialize, Deserialize)]
#[archive_attr(derive(CheckBytes))]
pub struct TaskLogA {
    tick: u32,
    vals: Vec<f64>,
}

impl TaskLogA {
    pub fn new() -> TaskLogA {
        TaskLogA {
            tick: 0,
            vals: vec![],
        }
    }

    pub fn add(&mut self, tick: u32, val: f64) {
        if self.tick != tick {
            self.vals.clear();
        }
        self.vals.push(val);
    }
}
#[derive(Archive, Serialize, Deserialize)]
#[archive_attr(derive(CheckBytes))]
pub struct TaskLogB {}

impl TaskLogB {
    pub fn new() -> TaskLogB {
        TaskLogB {}
    }
}

pub fn new_tasklog_as_enum() -> TaskLog {
    TaskLog::A(TaskLogA::new())
}

pub fn get_tasklog(borrow: &TaskLog) -> &TaskLogA {
    match borrow {
        TaskLog::A(a) => a,
        TaskLog::B(_) => panic!("DN"),
    }
}

pub fn get_mut_tasklog(borrow: &mut TaskLog) -> &mut TaskLogA {
    match borrow {
        TaskLog::A(a) => a,
        TaskLog::B(_) => panic!("DN"),
    }
}

pub fn taskdata_get(k: u16, largest: bool) -> f64 {
    MEMORY.with(|memory_refcell| {
        let borrow = &*memory_refcell.borrow();
        let memory_var = get_memory(borrow);
        let perf = &*memory_var.perf.borrow();
        let perf = get_perflog(perf);
        return match perf.perf_data.get(&k) {
            Some(val) => {
            let data = get_taskdata(val);
                data.get(largest)
            },
            None => 0f64,
        };
    })
}

// Disable/enable perf based on stuff - modify the macro but make it a software  later
#[macro_export]
macro_rules! perf {
    ( $name:expr, $($code:stmt);+ $(;)?) => {
        let begin = screeps::game::cpu::get_used();
        $(
            $code
        )*
        let end = screeps::game::cpu::get_used();
        crate::memory::MEMORY.with(|memory_refcell| {
            let borrow = memory_refcell.borrow();
            let memory = crate::memory::get_memory(&borrow);
            let mut borrow = memory.perf.borrow_mut();
            let perf = crate::performance::get_mut_perflog(&mut borrow);
            perf.add_val($name, end - begin);
        });
    };
}

// Task to TaskInt register:

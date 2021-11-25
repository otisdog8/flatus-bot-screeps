use std::borrow::BorrowMut;
use std::cell::RefCell;
use std::collections::HashSet;
use std::{collections::HashMap, rc::Rc};

use crate::ipc::{InterProcessCommunication, new_ipc_as_enum, get_mut_ipc};
use crate::process::ProcessContainer;
use crate::refcell_serialization::InlineRefCell;
use crate::scheduler::{new_scheduler_as_enum, Scheduler, get_mut_scheduler};

use bytecheck::CheckBytes;
use rkyv::{Archive, Deserialize, Serialize};
use rkyv_dyn::archive_dyn;
use screeps::game;

// Consts for CPU management
const BUCKET_MAX: u32 = (screeps::CPU_BUCKET_MAX as f64 * 0.95) as u32;
const BUCKET_OK: u32 = (screeps::CPU_BUCKET_MAX as f64 * 0.10) as u32;
const BUCKET_MIN: u32 = (screeps::CPU_BUCKET_MAX as f64 * 0.10) as u32;

// Kernel starts by getting a CPU budget. Hard limit and soft limits - hard limits = termination soft limits = need
// if CPU>9500 soft limit should be cpu or greater
// if less it should be less than cpu max
// Keeps lots of stuff behind RefCells so that it is commonly accessible

#[derive(Archive, Serialize, Deserialize)]
#[archive_attr(derive(CheckBytes))]
enum Kernel {
    A(KernelA),
    B(KernelB),
}

#[derive(Archive, Serialize, Deserialize)]
#[archive_attr(derive(CheckBytes))]
struct KernelA {
    #[with(InlineRefCell)]
    tick: RefCell<u32>,
    #[with(InlineRefCell)]
    scheduler: RefCell<Scheduler>,
    #[with(InlineRefCell)]
    ran_pids: RefCell<HashSet<u32>>,
    #[with(InlineRefCell)]
    processes: RefCell<HashMap<u32, ProcessContainer>>,
    #[with(InlineRefCell)]
    ipc: RefCell<InterProcessCommunication>,
    #[with(InlineRefCell)]
    pid: RefCell<u32>,
    #[with(InlineRefCell)]
    add_queue: RefCell<Vec<ProcessContainer>>,
    #[with(InlineRefCell)]
    del_queue: RefCell<Vec<u32>>,
}

impl KernelA {
    pub fn new() -> KernelA {
        let kernel = KernelA {
            tick: RefCell::new(0),
            scheduler: RefCell::new(new_scheduler_as_enum()),
            ran_pids: RefCell::new(HashSet::new()),
            processes: RefCell::new(HashMap::new()),
            ipc: RefCell::new(new_ipc_as_enum()),
            pid: RefCell::new(1),
            add_queue: RefCell::new(Vec::new()),
            del_queue: RefCell::new(Vec::new()),
        };
        // Start process with pid 1 (init+coordinator)

        kernel
    }

    pub fn init(&self) {
        // what runs after a reset
    }

    pub fn run_kernel(&self) {
        // Get CPU limit
        // Run Pretick
        // Pass CPU limit to tick function
        // Perflog pretick - but also perflog stuff inside pretick (like the scheduler pretick)

    }

    // Soft limit then hard limit
    pub fn get_cpu_limit(&self) -> (f64, f64) {
        let bucket=  game::cpu::bucket() as u32;
        if bucket > BUCKET_MAX {
            return (bucket as f64 * 0.05, 500f64);
        }

        if bucket < BUCKET_OK {
            return (game::cpu::limit() as f64*0.2, game::cpu::limit() as f64*0.5)
        }

        if bucket < BUCKET_MIN {
            return (game::cpu::limit() as f64*0.1, game::cpu::limit() as f64*0.3)
        }

        return (game::cpu::limit() as f64*0.5, game::cpu::limit() as f64*0.9);
    }

    pub fn pretick(&self) {
        // Prepare scheduler - calculate order

    }

    pub fn tick(&self, limit: f64) {
        while limit > game::cpu::get_used() {

        }
        //let mut process = process.as_ref().val.borrow_mut();
        // Quit if not enough budget
        //if taskdata_get(process.get_ptype(), true) * 2f64 > budget_remaining {
        //    return false;
        //}
        //perf!(process.get_ptype(),
        //    let result = process.run();
        //);
        //self.ran_pids.borrow_mut().insert(process.get_pid());
        //true
        // Give each process ipc, kernel, and some sort of cache/data store

    }

    pub fn posttick(&self) {
        // Handle add queue
        // Handle delete queue
        // Keep a borrow for the scheduler - reborrowing is stupid and not efficient
        let borrow = &mut *self.scheduler.borrow_mut();
        let scheduler = get_mut_scheduler(borrow);
        let processes = &mut *self.processes.borrow_mut();
        let add_queue = &mut *self.add_queue.borrow_mut();
        let del_queue = &mut *self.del_queue.borrow_mut();
        let borrow = &mut *self.ipc.borrow_mut();
        let ipc = get_mut_ipc(borrow);
        while !add_queue.is_empty() {
            let p = add_queue.pop();
            if let Some(p) = p {
                scheduler.spawn_process(&p);
                let pid = p.val.borrow().get_pid();
                processes.insert(pid, p);
            }
        }
        while !del_queue.is_empty() {
            let pid = del_queue.pop();
            if let Some(pid) = pid {
                // Remove from scheduler
                scheduler.kill_process(pid);
                // Remove from ptable
                if let Some(p) = processes.remove(&pid) {
                    let p = p.val.borrow_mut();
                    for c in p.get_child_processes() {
                        // Schedule children to be killed
                        del_queue.push(c);
                    }
                    // Kill self
                    p.kill();
                    // Remove IPC
                    ipc.rm_pid(pid);
                }
                // Kill code (recursively to children also)
                // Remove from scheduler
                // Remove from IPC
            }
        }
    }

    // Oh, it turns out kernels also handle process stuff. Who would've thought.
    pub fn add_process(&self, process: ProcessContainer) -> u32 {
        // Add the process to a queue
        let add_queue = &mut *self.add_queue.borrow_mut();
        let pid = &mut *self.pid.borrow_mut();
        let pid_save = pid.clone();
        *pid = *pid + 1;
        process.val.borrow_mut().set_pid(pid_save);
        add_queue.push(process);

        // Get the PID number, increment it on this
        // In the queue, handle scheduler process stuff (add it to all relevant datastructures)
        // Also add it to the process table
        pid_save
    }

    pub fn kill_process(&self, pid: u32) {
        let del_queue = &mut *self.del_queue.borrow_mut();
        del_queue.push(pid);
        // Queue it in the removal queue.
        // Run the kill code (that gives it to PID 1, which gives it to the cleanup daemon)
        // Removal code removes it from the scheduler - also removes any messages from IPC
    }
}

#[derive(Archive, Serialize, Deserialize)]
#[archive_attr(derive(CheckBytes))]
struct KernelB {}

impl KernelB {
    pub fn new() -> KernelB {
        KernelB {}
    }
}

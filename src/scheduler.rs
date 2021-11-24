// Features:
// Inter-process-communication
// Handles things like resuming from wait with can_run method
// HashMap of processes (or some other data struct) by priority

use std::{borrow::{Borrow, BorrowMut}, cell::RefCell, collections::{HashMap, HashSet, VecDeque}, ops::Add, rc::Rc};

use crate::{performance::taskdata_get, process::{Process, SerializeProcess}};
use crate::refcell_serialization::InlineRefCell;
use crate::perf;
use bytecheck::CheckBytes;
use rkyv::{Archive, Deserialize, Serialize};
use rkyv_dyn::archive_dyn;

#[derive(Archive, Serialize, Deserialize)]
#[archive_attr(derive(CheckBytes))]
pub enum Scheduler {
    A(SchedulerA),
    B(SchedulerB),
}

#[derive(Archive, Serialize, Deserialize)]
#[archive_attr(derive(CheckBytes))]
pub struct SchedulerA {
    #[with(InlineRefCell)]
    ran_pids: RefCell<HashSet<u32>>,
    #[with(InlineRefCell)]
    prio: RefCell<HashMap<u32, u8>>,
    #[with(InlineRefCell)]
    processes: RefCell<HashMap<u32, Rc<ProcessContainer>>>,
    #[with(InlineRefCell)]
    pqueue: RefCell<Vec<Rc<ProcessContainer>>>,
    #[with(InlineRefCell)]
    pid: RefCell<u32>,
}

#[derive(Archive, Serialize, Deserialize)]
#[archive_attr(derive(CheckBytes))]
pub struct ProcessContainer {
    #[with(InlineRefCell)]
    pub val: RefCell<Box<dyn SerializeProcess>>,
}

impl SchedulerA {
    pub fn new() -> SchedulerA {
        SchedulerA {
            ran_pids: RefCell::new(HashSet::new()),
            prio: RefCell::new(HashMap::new()),
            processes: RefCell::new(HashMap::new()),
            pqueue: RefCell::new(Vec::new()),
            pid: RefCell::new(0),
        }
    }

    pub fn pretick(&self) {
        // to make the pqueue first divide by prio then add to pqueue in prio order
        // Borrows ran_pids mutably, prio mutably, borrows processes immutably, borrows pqueue mutably
        // 255 >= Prio >= 128 = reserved priorities - shifting doesn't happen and can't shift up there
        // Anything less - shifting is automatic
        let mut pqueue_gen: Vec<Vec<Rc<ProcessContainer>>> = Vec::new();
        pqueue_gen.reserve(256);
        for i in 0..256 {
            pqueue_gen.append(&mut Vec::new());
        }
        for (k, v) in &*self.prio.borrow() {
            let arr = pqueue_gen.get_mut(*v as usize).unwrap();
            let process = self.get_process(*k).unwrap();
            if process.as_ref().val.borrow().as_ref().can_run() {
                arr.push(process)
            }
        }
        let pqueue = &mut *self.pqueue.borrow_mut();
        for i in 0..256 {
            pqueue.append(&mut pqueue_gen[i]);
        }
    }

    // true to continue, false to stop
    pub fn run_next_process(&self, budget_remaining: f64) -> bool {
        // Borrows pqueue mutably,
        let process = self.pqueue.borrow_mut().pop();
        let process = match process {
            Some(a) => a,
            None => return false
        };
        let mut process = process.as_ref().val.borrow_mut();
        let process = process.as_mut();
        // Quit if not enough budget
        if taskdata_get(process.get_ptype(), true) * 2f64 > budget_remaining {
            return false;
        }
        perf!(process.get_ptype(),
            let result = process.run();
        );
        self.ran_pids.borrow_mut().insert(process.get_pid());
        true
    }

    pub fn posttick(&self) {
        let pqueue = &mut *self.pqueue.borrow_mut();
        pqueue.clear();
        // shifting
        let mut ran = self.ran_pids.borrow_mut();
        for (k, v) in self.prio.borrow_mut().iter_mut() {
            if !ran.contains(k) {
                if *v < 127 {
                    *v += 1;
                }
            }
        }
        ran.clear();
    }

    pub fn spawn_process(&self, process: ProcessContainer) -> u16 {
        0
    }

    pub fn kill_process(&self, pid: u32) {

    }

    pub fn get_process(&self, pid: u32) -> Option<Rc<ProcessContainer>> {
        // borrows processes immutably
        let borrow = self.processes.borrow();
        match borrow.get(&pid) {
            Some(val) => Some(val.clone()),
            None => None,
        }
    }
}
#[derive(Archive, Serialize, Deserialize)]
#[archive_attr(derive(CheckBytes))]
pub struct SchedulerB {}

impl SchedulerB {
    pub fn new() -> SchedulerB {
        SchedulerB {}
    }
}

// Features:
// Inter-process-communication
// Handles things like resuming from wait with can_run method
// HashMap of processes (or some other data struct) by priority

use std::{
    borrow::{Borrow, BorrowMut},
    cell::RefCell,
    collections::{HashMap, HashSet, VecDeque},
    ops::Add,
    rc::Rc,
};

use crate::perf;
use crate::refcell_serialization::InlineRefCell;
use crate::{performance::taskdata_get, process::ProcessContainer};

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
    prio: HashMap<u32, u8>,
    pqueue: Vec<u32>,
}

impl SchedulerA {
    pub fn new() -> SchedulerA {
        SchedulerA {
            prio: HashMap::new(),
            pqueue: Vec::new(),
        }
    }

    pub fn pretick(&mut self) {
        // to make the pqueue first divide by prio then add to pqueue in prio order
        // Borrows ran_pids mutably, prio mutably, borrows processes immutably, borrows pqueue mutably
        // 255 >= Prio >= 128 = reserved priorities - shifting doesn't happen and can't shift up there
        // Anything less - shifting is automatic
        let mut pqueue_gen: Vec<Vec<u32>> = Vec::new();
        pqueue_gen.reserve(256);
        for i in 0..256 {
            pqueue_gen.append(&mut Vec::new());
        }
        for (k, v) in &self.prio {
            let arr = pqueue_gen.get_mut(*v as usize).unwrap();
            arr.push(*k);
        }
        for i in 0..256 {
            self.pqueue.append(&mut pqueue_gen[i]);
        }
    }

    // true to continue, false to stop
    pub fn run_next_process(&mut self) -> u32 {
        // Borrows pqueue mutably,
        let process = self.pqueue.pop();
        match process {
            Some(a) => return a,
            None => return 0,
        };
    }

    pub fn posttick(&mut self, ran: &mut HashSet<u32>) {
        self.pqueue.clear();
        // shifting
        for (k, v) in self.prio.iter_mut() {
            if !ran.contains(k) {
                if *v < 127 {
                    *v += 1;
                }
            }
        }
    }

    pub fn spawn_process(&mut self, process: &ProcessContainer) {

    }

    pub fn kill_process(&self, pid: u32) {}
}
#[derive(Archive, Serialize, Deserialize)]
#[archive_attr(derive(CheckBytes))]
pub struct SchedulerB {}

impl SchedulerB {
    pub fn new() -> SchedulerB {
        SchedulerB {}
    }
}

pub fn new_scheduler_as_enum() -> Scheduler {
    Scheduler::A(SchedulerA::new())
}

pub fn get_scheduler(borrow: &Scheduler) -> &SchedulerA {
    match borrow {
        Scheduler::A(a) => a,
        &Scheduler::B(_) => panic!("DN"),
    }
}

pub fn get_mut_scheduler(borrow: &mut Scheduler) -> &mut SchedulerA {
    match borrow {
        Scheduler::A(a) => a,
        Scheduler::B(_) => panic!("DN"),
    }
}

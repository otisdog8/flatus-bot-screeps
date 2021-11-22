use crate::memory;

// Here is where I log performance by averaging CPU deltas for processes
// Keep n deltas and move to a new delta every m, evict oldest deltas every once and a while too
// Index by hashmap and primitive strings - keep 2 deltas of 200 each (so a tuple)
// Keep this in Memory to aid scheduling, but offload to fs if bucket permitting (make fs offload a process with low prio)

pub struct PerfLog {

}
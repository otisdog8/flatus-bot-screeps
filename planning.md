# Planning for the stuff

Scheduler gets serialized and also handles everything pertaining to processes (processes, getprocesses, functions that add or delete processes).
Use sets to efficiently promote processes in priority - processes stored by pid in a datastructure that guarantees O(1) lookup via pid. Set of run processes, set of not run processes get iterated and promoted O(n)
running a process must be O(1), pre/posttick can be O(n) or nlogn at max

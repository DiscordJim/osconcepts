# osconcepts
Operating System Concepts implemented in Rust

# Implementations
1. Non-uniform memory (NUMA) is implemented simply using a random delay whenever we dereference into memory.
2. There is a full scheduler implementation.
3. There is a full multi-level feedback queue implementation.

# Examples
1. `asyn_dynamic.rs` Asymmetric multiprogramming with dynamic dispatching along with processor affinities and non-uniform memory access (NUMA)
2. `assymmp.rs` Asymmetric multiprogramming.
3. `numa.rs` Non-uniform memory access (NUMA).
4. `uma.rs` Uniform memory access (UMA)
5. `symmp.rs` Symmetric multiprogramming.
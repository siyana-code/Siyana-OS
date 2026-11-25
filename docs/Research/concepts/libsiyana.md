# libsiyana

The Rust library apps use to talk to Siyana. Same API, two backends: Linux (so apps run on Ubuntu today) and Siyana (raw syscalls). Apps never make syscalls themselves, so the ABI can evolve without breaking them.

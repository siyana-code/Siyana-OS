# ABI (Application Binary Interface)

The exact machine-level contract between programs and the OS: call numbers, which register holds what, struct layouts, error codes, executable format. Change it and every compiled program breaks, so Siyana's is versioned (docs/abi.md, crate siyana-abi).

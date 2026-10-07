# panic_handler

Rust requires one function marked #[panic_handler] for no_std. Ours just halts in a loop. Should later print a message and halt the CPU.

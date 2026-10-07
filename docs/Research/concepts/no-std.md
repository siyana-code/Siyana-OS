# no_std / no_main

Rust attributes. #![no_std] removes the standard library (which needs an OS). #![no_main] removes the C main() startup — we define _start ourselves.

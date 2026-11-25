# addr2line

A binutils tool that turns a code address into a function name (and file:line when debug info exists). Page faults log RIP; run `addr2line -f -e target/x86_64-unknown-none/release/siyana_os <RIP>` to see which function crashed.

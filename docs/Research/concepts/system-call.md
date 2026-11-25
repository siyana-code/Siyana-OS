# System call (syscall)

How a program asks the kernel to do something it can't do itself (read a file, start a process). On x86-64 the `syscall` instruction jumps into the kernel with the call number in rax and arguments in registers; the answer comes back in rax. Siyana's calls are listed in docs/abi.md.

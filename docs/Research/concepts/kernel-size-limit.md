# Kernel size limit

The kernel loads at 0x10000 and the boot stack sits just below 0x90000. linker.ld ASSERTs the kernel ends by 0x80000 (448 KiB), keeping 64 KiB for the stack, so the build fails instead of the kernel overwriting its own stack. Moving the kernel above 1 MiB lifts this later.

# .bss section

Where zero-initialized statics live. The ELF only records its size, so objcopy leaves it out of kernel.bin and the loader never writes it. _start zeroes the range __bss_start..__bss_end (labels from linker.ld) before anything else runs.

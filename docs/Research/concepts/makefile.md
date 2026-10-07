# Makefile

Recipe file: nasm assembles boot stages, cargo builds the kernel, objcopy makes kernel.bin, then boot.bin+stage2.bin+kernel.bin are concatenated into disk.img. Targets: make, make run, make clean.

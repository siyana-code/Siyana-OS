# Boot sector (Stage 1)

The first 512 bytes of the boot disk. BIOS requires it to end with magic bytes 0x55AA. It must be small, so its only job is to load the next, bigger stage (stage2.bin) into memory.

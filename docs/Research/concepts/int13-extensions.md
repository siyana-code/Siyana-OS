# INT 13h extensions (AH=41h / AH=42h)

The newer BIOS disk calls. AH=41h asks "do you support extended reads?". AH=42h reads sectors by LBA, described by a Disk Address Packet. Stage 2 calls it in a loop, 64 sectors (32 KiB) at a time, so the kernel can be far bigger than one old-style read allows.

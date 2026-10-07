# Port I/O (port 0x60)

x86 has 64K of I/O ports separate from RAM. Hardware listens on fixed ports. Port 0x60 holds the keyboard scancode. Reading it is required to re-arm IRQ1.

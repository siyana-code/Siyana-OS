# Serial port

The oldest PC way to send bytes out one at a time over a wire. Kernels love it for logging: it works before graphics, needs no driver stack, and survives crashes. Our driver is src/serial.rs; use serial_println! anywhere in the kernel.

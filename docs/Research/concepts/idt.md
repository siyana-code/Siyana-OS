# IDT (Interrupt Descriptor Table)

A table mapping interrupt numbers (0-255) to handler function addresses. Loaded with lidt. In src/interrupts.rs we register divide-by-zero and keyboard handlers.

# IRQ remapping

By default IRQ0-15 land on CPU vectors 0-15, colliding with exceptions. We reprogram the PICs so IRQ0-7 → vectors 32-39 (PIC1), IRQ8-15 → 40-47 (PIC2).

# Masked interrupts

Each PIC has a mask register; a 1 bit silences that IRQ. We mask everything except IRQ1 (keyboard) because BIOS leaves IRQ0 (timer) unmasked and we have no timer handler yet — it was triple-faulting us.

# Interrupt-safe locking

If code holds a spinlock and an interrupt handler tries to take the same lock, the handler spins forever: the code holding it can't run until the handler returns. Deadlock. So the heap lock is only taken with interrupts disabled (x86_64's without_interrupts).

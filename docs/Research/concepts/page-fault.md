# Page fault

CPU exception #14. Raised when a program touches a virtual address that isn't mapped, or with wrong permissions. Our handler prints the faulting address and cause, then halts.

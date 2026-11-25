# CR2 register

On a page fault the CPU stores the faulting virtual address in CR2. Our handler reads it via x86_64::registers::control::Cr2.

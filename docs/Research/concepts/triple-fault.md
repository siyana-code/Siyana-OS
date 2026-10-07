# Triple fault

CPU faults while handling a fault while handling a fault → it gives up and resets the machine. QEMU logs them with -d int. Usually means: unhandled interrupt or bad IDT entry.

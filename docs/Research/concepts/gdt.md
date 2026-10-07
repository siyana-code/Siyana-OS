# GDT (Global Descriptor Table)

A table describing code/data segments. Mostly vestigial in 64-bit mode but required to pass through protected mode and to define kernel vs user code segments. Loaded with lgdt.

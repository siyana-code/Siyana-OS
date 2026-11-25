ENTRY(_start)

SECTIONS
{
    . = 0x10000;

    /DISCARD/ : {
        *(.eh_frame)
        *(.eh_frame_hdr)
        *(.comment)
    }

    .text : {
        KEEP(*(.text.start))
        *(.text*)
    }

    .rodata : {
        *(.rodata*)
    }

    .data : {
        *(.data*)
    }

    .bss : {
        __bss_start = .;
        *(.bss*)
        __bss_end = .;
    }

    .got : {
        *(.got*)
    }

    /* Stage 2 sets the stack at 0x90000, growing down. Keep 64 KiB for it. */
    ASSERT(. <= 0x80000, "kernel too large: would overlap the boot stack at 0x80000")
}
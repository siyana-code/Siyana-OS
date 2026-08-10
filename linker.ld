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
        *(.bss*)
    }

    .got : {
        *(.got*)
    }
}
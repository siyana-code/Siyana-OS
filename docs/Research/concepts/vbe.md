# VBE / VESA BIOS Extensions

BIOS functions (int 0x10, AX=0x4Fxx) for graphics modes. We ask for mode info (0x4F01), then set the mode (0x4F02). This gives us a linear framebuffer.

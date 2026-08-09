[org 0x8000]
[bits 16]

start_stage2:
    mov si, msg2

print2:
    lodsb
    cmp al, 0
    je switch_to_pm
    mov ah, 0x0E
    int 0x10
    jmp print2

switch_to_pm:
    cli
    lgdt [gdt_descriptor]

    mov eax, cr0
    or eax, 1
    mov cr0, eax

    jmp CODE_SEG:init_pm

[bits 32]
init_pm:
    mov ax, DATA_SEG
    mov ds, ax
    mov es, ax
    mov fs, ax
    mov gs, ax
    mov ss, ax

    mov ebp, 0x90000
    mov esp, ebp

    call protected_mode_main

CODE_SEG equ gdt_code - gdt_start
DATA_SEG equ gdt_data - gdt_start

msg2: db 0x0D, 0x0A, 'Stage 2 loaded and running!', 0

gdt_start:
    dq 0x0000000000000000

gdt_code:
    dw 0xFFFF
    dw 0x0000
    db 0x00
    db 10011010b
    db 11001111b
    db 0x00

gdt_data:
    dw 0xFFFF
    dw 0x0000
    db 0x00
    db 10010010b
    db 11001111b
    db 0x00

gdt_end:

gdt_descriptor:
    dw gdt_end - gdt_start - 1
    dd gdt_start

protected_mode_main:
    jmp $
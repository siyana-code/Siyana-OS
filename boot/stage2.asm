[org 0x8000]
[bits 16]

start_stage2:
    mov si, msg2

print2:
    lodsb
    cmp al, 0
    je load_kernel
    mov ah, 0x0E
    int 0x10
    jmp print2

load_kernel:
    mov ax, 0x1000
    mov es, ax
    mov bx, 0x0000

    mov ah, 0x02
    mov al, 17
    mov ch, 0
    mov cl, 26
    mov dh, 0
    int 0x13
    jc disk_error2

    ; --- Set VBE mode 0x112 (640x480, 24bpp) with linear framebuffer ---
    xor ax, ax
    mov es, ax
    mov ds, ax
    mov ax, 0x4F01          ; VBE: get mode info
    mov cx, 0x0112
    mov di, 0x7000          ; ModeInfoBlock buffer at physical 0x7000
    int 0x10
    cmp ax, 0x004F
    jne vbe_error
    mov eax, [0x7000 + 0x28] ; linear framebuffer base (bytes 0x28-0x2B)
    mov [0x7C00], eax
    mov ax, [0x7000 + 0x10]  ; pitch (bytes per scanline)
    mov [0x7C04], ax
    mov ax, [0x7000 + 0x12]  ; width
    mov [0x7C08], ax
    mov ax, [0x7000 + 0x14]  ; height
    mov [0x7C0A], ax

    mov ax, 0x4F02          ; VBE: set mode
    mov bx, 0x4112          ; mode 0x112 | linear fb bit (0x4000)
    int 0x10
    cmp ax, 0x004F
    jne vbe_error

    jmp switch_to_pm

vbe_error:
    mov si, vbe_msg
print_vbe_err:
    lodsb
    cmp al, 0
    je hang_vbe
    mov ah, 0x0E
    int 0x10
    jmp print_vbe_err

hang_vbe:
    hlt
    jmp hang_vbe

disk_error2:
    mov si, err_msg2

print_err2:
    lodsb
    cmp al, 0
    je hang2
    mov ah, 0x0E
    int 0x10
    jmp print_err2

hang2:
    hlt
    jmp hang2

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
CODE64_SEG equ gdt_code64 - gdt_start
DATA64_SEG equ gdt_data64 - gdt_start

msg2: db 0x0D, 0x0A, 'Stage 2 loaded and running!', 0
err_msg2: db 'Kernel disk read error!', 0
vbe_msg: db 'VBE framebuffer setup failed!', 0

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

gdt_code64:
    dw 0x0000
    dw 0x0000
    db 0x00
    db 10011010b
    db 00100000b
    db 0x00

gdt_data64:
    dw 0x0000
    dw 0x0000
    db 0x00
    db 10010010b
    db 0x00
    db 0x00

gdt_end:

gdt_descriptor:
    dw gdt_end - gdt_start - 1
    dd gdt_start

protected_mode_main:
    mov esi, pm_msg
    mov edi, 0xB8000
    mov ah, 0x0F

print_pm:
    mov al, [esi]
    cmp al, 0
    je enable_long_mode
    mov [edi], ax
    add esi, 1
    add edi, 2
    jmp print_pm

enable_long_mode:
    mov eax, cr4
    or eax, 1 << 5
    mov cr4, eax

    mov eax, pml4_table
    mov cr3, eax

    mov ecx, 0xC0000080
    rdmsr
    or eax, 1 << 8
    wrmsr

    mov eax, cr0
    or eax, 1 << 31
    mov cr0, eax

    jmp CODE64_SEG:long_mode_start

[bits 64]
long_mode_start:
    mov ax, DATA64_SEG
    mov ds, ax
    mov es, ax
    mov fs, ax
    mov gs, ax
    mov ss, ax

    mov rsi, lm_msg
    mov rdi, 0xB80A0
    mov ah, 0x0F

print_lm:
    mov al, [rsi]
    cmp al, 0
    je lm_hang
    mov [rdi], ax
    add rsi, 1
    add rdi, 2
    jmp print_lm

lm_hang:
    jmp 0x10000

pm_msg: db 'Protected mode active - Siyana OS', 0
lm_msg: db 'Long mode active - 64-bit CPU engaged', 0

align 4096
pml4_table:
    dq pdpt_table + 0x03
    times 511 dq 0

align 4096
pdpt_table:
    dq 0x00000083        ; 0–1 GB identity mapped
    dq 0
    dq 0x80000083        ; 2–3 GB identity mapped
    dq 0xC0000083        ; 3–4 GB identity mapped (VBE LFB lives here)
    times 508 dq 0
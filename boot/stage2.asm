[org 0x8000]
[bits 16]

start_stage2:
    mov si, msg2

print2:
    lodsb
    cmp al, 0
    je hang
    mov ah, 0x0E
    int 0x10
    jmp print2

hang:
    hlt
    jmp hang

msg2: db 'Stage 2 loaded and running!', 0
[org 0x7C00]
[bits 16]

start:
    mov si, message

print_char:
    lodsb
    cmp al, 0
    je load_stage2
    mov ah, 0x0E
    int 0x10
    jmp print_char

load_stage2:
    mov bx, 0x8000
    mov ah, 0x02
    mov al, 1
    mov ch, 0
    mov cl, 2
    mov dh, 0
    int 0x13
    jc disk_error

    jmp 0x8000

disk_error:
    mov si, err_msg

print_error:
    lodsb
    cmp al, 0
    je hang
    mov ah, 0x0E
    int 0x10
    jmp print_error

hang:
    hlt
    jmp hang

message: db 'Siyana OS booting...', 0
err_msg: db 'Disk read error!', 0

times 510-($-$$) db 0
dw 0xAA55
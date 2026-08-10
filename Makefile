.PHONY: all clean run

BOOT_BIN = boot/boot.bin
STAGE2_BIN = boot/stage2.bin
KERNEL_ELF = target/x86_64-unknown-none/release/siyana_os
KERNEL_BIN = boot/kernel.bin
DISK_IMG = boot/disk.img

all: $(DISK_IMG)

$(BOOT_BIN): boot/boot.asm
	nasm -f bin boot/boot.asm -o $(BOOT_BIN)

$(STAGE2_BIN): boot/stage2.asm
	nasm -f bin boot/stage2.asm -o $(STAGE2_BIN)

$(KERNEL_ELF): src/main.rs linker.ld
	cargo build --release

$(KERNEL_BIN): $(KERNEL_ELF)
	objcopy -O binary $(KERNEL_ELF) $(KERNEL_BIN)
	truncate -s 7680 $(KERNEL_BIN)

$(DISK_IMG): $(BOOT_BIN) $(STAGE2_BIN) $(KERNEL_BIN)
	cat $(BOOT_BIN) $(STAGE2_BIN) $(KERNEL_BIN) > $(DISK_IMG)

run: $(DISK_IMG)
	qemu-system-x86_64 -drive format=raw,file=$(DISK_IMG) -no-reboot

clean:
	rm -f $(BOOT_BIN) $(STAGE2_BIN) $(KERNEL_BIN) $(DISK_IMG)
	cargo clean
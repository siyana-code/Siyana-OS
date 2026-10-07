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

$(KERNEL_ELF): src/main.rs src/interrupts.rs linker.ld
	cargo build --release

$(KERNEL_BIN): $(KERNEL_ELF)
	objcopy -O binary $(KERNEL_ELF) $(KERNEL_BIN)
	@SIZE=$$(stat -c%s $(KERNEL_BIN)); \
	SECTORS=$$(( (SIZE + 511) / 512 )); \
	PADDED=$$(( SECTORS * 512 )); \
	truncate -s $$PADDED $(KERNEL_BIN); \
	echo "Kernel: $$SIZE bytes -> padded to $$PADDED bytes ($$SECTORS sectors)"; \
	echo $$SECTORS > boot/.kernel_sectors

$(DISK_IMG): $(BOOT_BIN) $(STAGE2_BIN) $(KERNEL_BIN)
	@SECTORS=$$(cat boot/.kernel_sectors); \
	CURRENT=$$(grep -oP '(?<=mov al, )\d+' boot/stage2.asm | tail -1); \
	if [ "$$SECTORS" != "$$CURRENT" ]; then \
		echo "WARNING: stage2.asm reads $$CURRENT sectors but kernel needs $$SECTORS. Update boot/boot.asm's 'mov al, N' under load_kernel."; \
		exit 1; \
	fi
	cat $(BOOT_BIN) $(STAGE2_BIN) $(KERNEL_BIN) > $(DISK_IMG)

run: $(DISK_IMG)
	qemu-system-x86_64 -drive format=raw,file=$(DISK_IMG) -no-reboot

clean:
	rm -f $(BOOT_BIN) $(STAGE2_BIN) $(KERNEL_BIN) $(DISK_IMG) boot/.kernel_sectors
	cargo clean
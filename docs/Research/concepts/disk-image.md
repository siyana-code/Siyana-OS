# Disk image

boot/disk.img — a file that is a fake hard disk. We concatenate boot.bin + stage2.bin + kernel.bin. QEMU boots it exactly like a real disk. It is a build artifact, so it is git-ignored.

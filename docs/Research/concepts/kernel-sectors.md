# Kernel sectors

The kernel binary's size divided by 512, rounded up (padded). The Makefile computes it and passes it to NASM as KERNEL_SECTORS when it assembles stage 2, so stage 2 always loads exactly the whole kernel. Nothing to edit by hand.

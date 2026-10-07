# Kernel sectors

The kernel binary's size divided by 512, rounded up (padded). stage2.asm hardcodes how many sectors to read; the Makefile computes it and FAILS the build if they disagree, so the kernel never loads truncated.

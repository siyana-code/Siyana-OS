# LBA (Logical Block Addressing)

Numbering disk sectors 0, 1, 2, ... in one straight line. The old way (CHS: cylinder/head/sector) needs disk geometry and breaks at track edges. Our kernel starts at LBA 25: sector 0 is stage 1, sectors 1-24 are stage 2.

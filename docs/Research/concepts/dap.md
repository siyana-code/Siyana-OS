# Disk Address Packet (DAP)

A 16-byte note we hand to INT 13h AH=42h: packet size, how many sectors, where in memory (segment:offset), and which LBA to start at. Stage 2 rewrites it before each chunk.

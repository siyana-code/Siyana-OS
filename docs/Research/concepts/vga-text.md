# VGA text buffer (0xB8000)

Writing bytes to physical 0xB8000 shows them as text. Each character = 2 bytes: ASCII + attribute. Our early kernel printed this way.

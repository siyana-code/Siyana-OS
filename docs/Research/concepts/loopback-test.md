# Loopback test

The UART can wire its output straight back to its input. We send 0xAE in loopback mode and check it comes back. If it doesn't, there is no working UART, and logging quietly turns itself off instead of hanging.

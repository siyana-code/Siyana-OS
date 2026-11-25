# Baud rate divisor and DLAB

The UART's clock runs at 115200 ticks per second; you divide it to pick a speed. Setting the DLAB bit in the line control register turns the first two ports into the divisor's low/high bytes. We write divisor 3, giving 38400 baud, then clear DLAB.

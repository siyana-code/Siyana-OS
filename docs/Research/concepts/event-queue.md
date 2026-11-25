# Event queue (instead of signals)

Unix signals interrupt a program at a random moment and run a handler that may only do a few 'safe' things. Siyana instead queues events (Shutdown, ChildExited, Readable) and the program picks them up in its main loop with event_wait. Same information, no surprise interruptions.

# Zombie process

A process that has exited but whose parent hasn't collected its exit status yet. It uses no CPU, just a slot in the process table. wait/reap clears it.

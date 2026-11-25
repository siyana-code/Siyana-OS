# Emergency printing (force_unlock)

A panic or page fault can happen while the serial lock is held, and the holder will never run again. So the crash path breaks the lock (spin's force_unlock) and prints anyway. Only safe because nothing else runs after a crash.

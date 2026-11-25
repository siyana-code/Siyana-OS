# Reaper (subreaper)

When a process dies, its children become orphans. They are handed to the nearest ancestor marked as a reaper (init at the top), which collects their exit status with wait so they don't linger as zombies.

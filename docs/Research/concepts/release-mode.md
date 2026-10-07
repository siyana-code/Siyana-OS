# Release mode

cargo build --release compiles optimized, without debug-only runtime checks (like stack probes) that a freestanding kernel can't satisfy. Debug builds won't boot.

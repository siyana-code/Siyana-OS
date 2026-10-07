# Framebuffer driver (src/fb.rs)

Our module: reads the LFB address/pitch/size from 0x7C00 and provides put_pixel, clear, draw_text. This is the base layer of the future GUI.

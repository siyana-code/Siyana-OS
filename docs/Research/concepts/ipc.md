# IPC (inter-process communication)

Ways for processes to talk. Siyana v0 has named streams: a server listens on a name, clients connect to it, both read and write bytes. On Linux, libsiyana maps names to Unix domain sockets.

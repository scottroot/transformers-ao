#include <stdio.h>
#include <stdlib.h>
#include <fcntl.h>
#include <unistd.h>
#include <string.h>
#include <emscripten.h>

// Logging macro for debugging purposes. Set to 1 to enable logging.
#if 0
    #define AO_LOG(...) fprintf(stderr, __VA_ARGS__)
#else
    #define AO_LOG(...)
#endif

// Asynchronous function to open a file using WeaveDrive in the WebAssembly module.
EM_ASYNC_JS(int, weavedrive_open, (const char* c_filename, const char* mode), {
    const filename = UTF8ToString(c_filename);

    if (!Module.WeaveDrive) {
        console.error('WeaveDrive module not found');
        return Promise.resolve(null);
    }

    const drive = Module.WeaveDrive(Module, FS);

    try {
        const fd = await drive.open(filename);
        return fd;
    } catch (error) {
        console.error('Error opening file:', error);
        return null;
    }
});

// Asynchronous function to read from a file using WeaveDrive in the WebAssembly module.
EM_ASYNC_JS(int, weavedrive_read, (int fd, int *dst_ptr, size_t length), {
    if (!Module.WeaveDrive) {
        console.error('WeaveDrive module not found');
        return Promise.resolve(null);
    }

    const drive = Module.WeaveDrive(Module, FS);

    try {
        const bytesRead = await drive.read(fd, dst_ptr, length);
        return bytesRead;
    } catch (error) {
        console.error('Error reading file:', error);
        return null;
    }
});
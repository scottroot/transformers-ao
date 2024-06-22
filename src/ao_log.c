#include <emscripten.h>


EM_JS(void, ao_log_js, (const char* message), {
    const logMessage = UTF8ToString(message);
    if (typeof process !== 'undefined' && process.env && process.env.DEBUG) {
        console.log(logMessage);
    }
});
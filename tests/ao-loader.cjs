const Emscripten = require('../outtest.js')

module.exports = async function (binary, options) {
    let instance = await Emscripten(binary, options)
    let doHandle = instance.cwrap('handle', 'string', ['string', 'string'])

    if (instance?.cleanupListeners) {
        instance.cleanupListeners()
    }

    return async (buffer, msg, env) => {
        const originalRandom = Math.random
        const originalLog = console.log
        try {
            Math.random = function () { return 0.5 }
            // console.log = function () { return null }

            if (buffer) {
                if (instance.HEAPU8.byteLength < buffer.byteLength) {
                    console.log("RESIZE HEAP")
                    instance.resizeHeap(buffer.byteLength)
                }
                instance.HEAPU8.set(buffer)
            }

            const res = await doHandle(JSON.stringify(msg), JSON.stringify(env));
            // console.log("\n\n-----------------------------\nRES:");
            // console.log(String(res));
            // console.log("-----------------------------\n\n");

            const { ok, response } = JSON.parse(res)
            if (!ok) throw response

            Math.random = originalRandom
            console.log = originalLog
            /** end unmock */

            return {
                Memory: instance.HEAPU8.slice(),
                Error: response.Error,
                Output: response.Output,
                Messages: response.Messages,
                Spawns: response.Spawns,
                Assignments: response.Assignments,
                GasUsed: instance?.gas?.used || 0
            }
        } finally {
            // eslint-disable-next-line no-global-assign
            // Date = OriginalDate
            Math.random = originalRandom
            console.log = originalLog
            buffer = null
        }
    }
}
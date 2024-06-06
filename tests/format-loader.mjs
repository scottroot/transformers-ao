import fs from "fs";

let filePath = process.argv[2];
let data = fs.readFileSync(filePath, 'utf-8');

const functionModuleLine = `  return (
function(moduleArg = {}) {`;
const functionModuleReplacement = `return (function (binaryOrInstantiate, { computeLimit, memoryLimit, extensions, format }) {
    var Module = Module || {};
    if (typeof binaryOrInstantiate === "function")
      Module.instantiateWasm = binaryOrInstantiate;
    else Module.wasmBinary = binaryOrInstantiate;

    /**
     * See this issue with emscripten https://github.com/emscripten-core/emscripten/issues/12740
     *
     * We need to manually cleanup any listeners that are setup as part of the WASM module,
     * so that they can be deregistered later and the associated WASM memory can be garbage collected
     *
     * This is custom code we've added to the emscripten module code.
     */
    const _listeners_ = [];
    Module.cleanupListeners = function () {
      /**
       * Deregister any listeners that did not exist before this
       * WASM module was bootstrapped
       */
      _listeners_.forEach(([name, l]) => process.removeListener(name, l));
    };
    function uncaughtException(ex) {
      if (!(ex instanceof ExitStatus)) {
        throw ex;
      }
    }
    function unhandledRejection(reason) {
      throw reason;
    }
    _listeners_.push(
      ["uncaughtException", uncaughtException],
      ["unhandledRejection", unhandledRejection]
    );`;
data = data.replace(functionModuleLine, functionModuleReplacement);


data = data.replace("var Module = moduleArg;", "");
data = data.replace("var Module=moduleArg;", "");


// const wasiSnapshotLine = `'wasi_snapshot_preview1': wasmImports,`;
// const wasiSnapshotReplacement = `"wasi_snapshot_preview1": wasmImports,
//         metering: { usegas: function (gas) { Module.gas.use(gas); if (Module.gas.isEmpty()) throw Error('out of gas!') } },`;
// // data = data.replace(wasiSnapshotLine, wasiSnapshotReplacement);
//
// data = data.replace(/['"]wasi_snapshot_preview1['"]:\s?wasmImports[,]?/, wasiSnapshotReplacement);


const moduleReadyLine = `  return moduleArg.ready`;
const moduleReadyReplacement = `/**
 * Expose the ability to resize the WASM heap.
 *
 * The WASM heap is set to auto-grow, but starts with an initial small size.
 * If we try to load a previously obtained heap, that is larger than the initial size
 * due to it having been auto-grown, we will receive an RangeError due to the initial
 * size being too small to store our heap
 *
 * Exposing resize_heap allows us to expand the initial size, if needed, before loading in our heap.
 */
    Module.resizeHeap = _emscripten_resize_heap;

    return Module.ready;`;
data = data.replace(moduleReadyLine, moduleReadyReplacement);

fs.writeFile(filePath, data, 'utf8', (err) => {
    if (err) {
        console.error('Error writing the file:', err);
    } else {
        console.log('File has been updated');
    }
});
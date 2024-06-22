import fs from "fs";

let filePath = process.argv[2];
let data = fs.readFileSync(filePath, 'utf-8');

data = `const DEFAULT_GAS_LIMIT = 9_000_000_000_000_000;\n${data}`;


data = data.replace(
  /var Module\s?=\s?moduleArg;/,
  `var Module = moduleArg;
      Module.gas = {
        limit: Module?.computeLimit || DEFAULT_GAS_LIMIT,
        used: 0,
        use: (amount) => {
          Module.gas.used += amount;
        },
        refill: (amount) => {
          if (!amount) Module.gas.used = 0;
          else Module.gas.used = Math.max(Module.gas.used - amount, 0);
        },
        isEmpty: () => Module.gas.used > Module.gas.limit,
      };
`);

// const readyOriginal = `var readyPromiseResolve, readyPromiseReject;`;
// const readyReplace = `var readyPromiseResolve, readyPromiseReject;
//       var readyPromise = new Promise((resolve, reject) => {
//         readyPromiseResolve = resolve;
//         readyPromiseReject = reject;
//       });
// `
// data = data.replace(readyOriginal, readyReplace);

// data = data.replace("var Module = moduleArg;", "");
// data = data.replace("var Module=moduleArg;", "");


// const wasiSnapshotLine = `'wasi_snapshot_preview1': wasmImports,`;
// const wasiSnapshotReplacement = `"wasi_snapshot_preview1": wasmImports,
//         metering: { usegas: function (gas) { Module.gas.use(gas); if (Module.gas.isEmpty()) throw Error('out of gas!') } },`;
// // data = data.replace(wasiSnapshotLine, wasiSnapshotReplacement);

// data = data.replace(/['"]wasi_snapshot_preview1['"]:\s?wasmImports[,]?/, wasiSnapshotReplacement);

data = data.replace(
  /var info\s?=\s?{/,
  `var info = {
    metering: { usegas: function (gas) { Module.gas.use(gas); if (Module.gas.isEmpty()) throw Error('out of gas!') } },
`);

data = data.replace("return moduleArg.ready", `
  Module.resizeHeap = _emscripten_resize_heap;
  return moduleArg.ready
`);
fs.writeFile(filePath.replace(".js", ".cjs"), data, 'utf8', (err) => {
    if (err) {
        console.error('Error writing the file:', err);
    } else {
        console.log('File has been updated');
    }
});
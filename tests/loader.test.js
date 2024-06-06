import fs from "fs";
import {default as AoLoader} from "./ao-loader.cjs";


const env = {Process: {Id: 'AOS', Owner: 'FOOBAR', Tags: [{ name: 'Name', value: 'Thomas' }]}}

async function runWasmModule() {
  const wasmBinary = fs.readFileSync("outtest.wasm");
  const handle = await AoLoader(wasmBinary, { format: "wasm32-unknown-emscripten4" })

  const result1 = await handle(null, {
    Target: 'AOS', Owner: 'FOOBAR', ['Block-Height']: "1000", Id: "1234xyxfoo1", Module: "WOOPAWOOPA",
    Tags: [{ name: 'Model-Type', value: 'bert' }],
    Data: `{name = "Bob"}`
  }, env);
  console.log(
    "Response1 from WASM:", 
    JSON.stringify({error: result1.Error, output: result1.Output, messages: result1.Messages, spawns: result1.Spawns, assignments: result1.Assignments})
  );
  console.log(result1);

  const result2 = await handle(result1.Memory, {
    Target: 'AOS', Owner: 'FOOBAR', ['Block-Height']: "1002", Id: "1234xyxfoo2", Module: "WOOPAWOOPA",
    Tags: [{ name: 'Action', value: 'inc' }],
    Data: ""
  }, env);
  console.log(
    "Response2 from WASM:", 
    JSON.stringify({error: result2.Error, output: result2.Output, messages: result2.Messages, spawns: result2.Spawns, assignments: result2.Assignments})
  );

  const result3 = await handle(result2.Memory, {
    Target: 'AOS', Owner: 'FOOBAR', ['Block-Height']: "1003", Id: "1234xyxfoo3", Module: "WOOPAWOOPA",
    Tags: [{ name: 'Action', value: 'inc' }],
    Data: ""
  }, env);
  console.log(
    "Response3 from WASM:",
    JSON.stringify({error: result3.Error, output: result3.Output, messages: result3.Messages, spawns: result3.Spawns, assignments: result3.Assignments})
  );

}

runWasmModule()

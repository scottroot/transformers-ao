import { readFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import {default as AoLoader} from "./ao-loader/index.cjs";
import weaveDrive from "./ao-loader/weavedrive.cjs";

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);


async function main() {
  if(!(await fetch("http://host.docker.internal:4000/info").then(r => r.ok))) throw("Arlocal not running");
  const wasmPath = await resolve(__dirname, "..", "build/transformers_ao.wasm");
  const wasmBinary = await readFileSync(wasmPath);
  const handle = await AoLoader((imports, cb) =>
    WebAssembly.instantiate(wasmBinary, imports).then((result) => cb(result.instance)), {
      format: 'wasm32-unknown-emscripten4',
      WeaveDrive: weaveDrive,
      admissableList: [
        "3EsHLtzhLU6nikyJfcfczlPpmp7PKD6L-wRSaVKpnlY", // that mp3 file
        "c1srK3dRQ9qWgSopUSozuPcCiuJvVK9uXhETZb4bsH4", // colors.json
      ],
      ARWEAVE: 'http://host.docker.internal:4000',
      mode: "test",
      blockHeight: 100,
      spawn: { "Scheduler": "AnTgDyREHJYwOi-6Kr3izJwLt9FLWMr7ZD_2NDhYj5k" },
      process: {
        id: "Fz-ml_KVzZswLB7ReqvAQ4QC-SM0Wb1CAz_AuryiwbA",  // TEST_PROCESS_ID
        owner: "OmO8vDHqD07rvl__qs1PGN3vPYtmtXJhFxVgayqrF2Q",  // TEST_PROCESS_OWNER
        tags: [
          { name: "Extension", value: "Weave-Drive" }
        ]
      }
    }
  );
  const txToUse = "c1srK3dRQ9qWgSopUSozuPcCiuJvVK9uXhETZb4bsH4";
  const { Memory, Output, Messages, Error } = await handle(null,
    {
      "Id": "FOO",
      "Owner": "tom",
      "Target": "AOS",
      "Tags": [
        { "name": "Action", "value": "Eval" }
      ],
      "Data": `
local wd = require("weavedrive")
local data = wd.open("/data/${txToUse}")
local data = wd.read(data)
return print(data)
      `,
      "Module": "1234",
      "Block-Height": "1000"
    },
    {
      "Process": { "Id": "ctr-id-456", "Tags": [] }
    }
  )
  console.log("\n", "*".repeat(79), "\n");
  console.log(JSON.stringify({Output: {...Output, prompt: undefined}}, null, 2))
}
main();

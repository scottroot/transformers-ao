import fs from "fs";
import {default as AoLoader} from "./ao-loader/index.cjs";
import weaveDrive from "./weavedrive/weavedrive.cjs";


const ENV = {Process: {Id: 'AOS', Owner: 'FOOBAR', Tags: [{ name: 'Name', value: 'Thomas' }]}}
const MSG = {
  "Target": "AOS",
  "Owner": "Foobar",
  "Id": "AdbSbdSB",
  "Module": "WOOOP",
  "Block-Height": "1000",
  Tags: [{ name: 'Action', value: 'eval' }],
  Data: `
  local wd = require("weavedrive")
  local data = wd.get_tx("3EsHLtzhLU6nikyJfcfczlPpmp7PKD6L-wRSaVKpnlY")
  return print(data)
`
}
async function main() {
  const wasmBinary = fs.readFileSync("../outtest.wasm");
  const handle = await AoLoader((imports, cb) =>
    WebAssembly.instantiate(wasmBinary, imports).then((result) => cb(result.instance)), {
      format: 'wasm32-unknown-emscripten4',
      WeaveDrive: weaveDrive,
      admissableList: [
        "dx3GrOQPV5Mwc1c-4HTsyq0s1TNugMf7XfIKJkyVQt8", // Random NFT metadata (1.7kb of JSON)
        "XOJ8FBxa6sGLwChnxhF2L71WkKLSKq1aU5Yn5WnFLrY", // GPT-2 117M model.
        "M-OzkyjxWhSvWYF87p0kvmkuAEEkvOzIj4nMNoSIydc", // GPT-2-XL 4-bit quantized model.
        "kd34P4974oqZf2Db-hFTUiCipsU6CzbR6t-iJoQhKIo", // Phi-2 
        "ISrbGzQot05rs_HKC08O_SmkipYQnqgB1yC3mjZZeEo", // Phi-3 Mini 4k Instruct
        "sKqjvBbhqKvgzZT4ojP1FNvt4r_30cqjuIIQIr-3088", // CodeQwen 1.5 7B Chat q3
        "Pr2YVrxd7VwNdg6ekC0NXWNKXxJbfTlHhhlrKbAd1dA", // Llama3 8B Instruct q4
        "jbx-H6aq7b3BbNCHlK50Jz9L-6pz9qmldrYXMwjqQVI"  // Llama3 8B Instruct q8      
      ],
      // ARWEAVE: 'http://localhost:4000',
      ARWEAVE: 'https://arweave.net',
      mode: "test",
      blockHeight: 100,
      spawn: { "Scheduler": "TEST_SCHED_ADDR" },
      process: {
        id: "TEST_PROCESS_ID",
        owner: "TEST_PROCESS_OWNER",
        tags: [
          { name: "Extension", value: "Weave-Drive" }
        ]
      }
    }
  )

  const r = await handle(null,
    {
      Id: 'FOO',
      Owner: 'tom',
      Target: 'AOS',
      Tags: [
        { name: 'Action', value: 'Eval' }
      ],
      Data: `
  local wd = require("weavedrive")
  local data = wd.open("/data/dx3GrOQPV5Mwc1c-4HTsyq0s1TNugMf7XfIKJkyVQt8")
  local data = wd.read(data)
  print(data == nil)
  return print(data)
      `,
      Module: '1234',
      ['Block-Height']: '1000'
    },
    {
      Process: { Id: 'ctr-id-456', Tags: [] }
    }
  )
  console.log(r)

  // const result = await handle(r.Memory,
  //   {
  //     Id: 'FOO',
  //     Owner: 'nottom',
  //     Target: 'AOS',
  //     Tags: [
  //       { name: 'Action', value: 'Eval' }
  //     ],
  //     Data: `
  // local result = ""
  // for i = 0, 100, 1 do
  //   local token = Llama.next()
  //   result = result .. token
  // end
  // return result
  //       `,
  //     Module: '1234',
  //     ['Block-Height']: '1000'
  //   },
  //   {
  //     Process: { Id: 'ctr-id-456', Tags: [] }
  //   }
  // )
  // console.log(result)
}
main()

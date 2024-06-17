![Transformers-AO Logo](web/images/transformers-ao.png)
# Transformers-AO (wip)
Transformers-AO brings the functionality of the popular [Hugging Face](https://huggingface.co/)'s Transformers library to Arweave's new cloud computing blockchain "AO".  
This means the ability to use the latest and your favorite open-source models in a familiar API.

----
## Components of Transformers-AO
 * WeaveDrive: method to deterministically load model/config data into AO processes
 * Model-Specific functions.  Currently live:
   * Bert

## Models supported
✅ SentenceTransformers (bert)  
⬜ Clip vit-base32 text model  
✅ Stable Diffusion 1.5  
⬜ Mistral  
⬜ Stable Diffusion 2  
⬜ Stable Diffusion XL  
⬜ Stable Diffusion Turbo  
✅ T5  
⬜ MusicGen (in progress)  
⬜ ...  

## What is AO?
> **TL;DR:** AO is a massive, shared-computer where tasks run in their own virtual machines. Requests and results are stored immutably on-chain with Arweave. Highly customizable, it is ideal for creating powerful machine learning pipelines.

AO is like a massive shared computer that anyone can use. Each task run on AO is executed in its own personal virtual machine, separate from anyone else’s.

To run a program or trigger a function, like a feature-extraction pipeline or generating an image with StableDiffusion, you open an AO instance and send a message listing the target machine (the “Process”), and tags indicating what you want to do. The results are computed, saved on-chain, and handled or returned based on your function's logic.

Because AO processes are highly customizable and easy to configure using a simple language called Lua, developers can create chained sequences that conditionally run based on the output of the previous result. This is similar to directed acyclic graphs (DAGs), and enables users to build powerful machine learning pipelines, either running synchronously or triggering many processes in parallel.

## How To Use
There are many ways to use TFAO both on-demand and as part of an existing AO/Arweave application.

### Primary tools to work with AO:
 * [AOconnect-ts](https://github.com/scottroot/aoconnect): Typescript package providing the basic tools to connect to AO, run tasks and read results
 * [AO Connect (official)](https://github.com/permaweb/ao/connect): Original JavaScript aoconnect npm package - same functionality as the TS version above, though likely to be more up-to-date.
 * [AOS](https://github.com/permaweb/aos): Command-line program to interact with AO within a Lua programming language runtime, send messages and read results.

Use the ao-connect package to call an AO process running the model you want to use
For example:
```JS
import { connect } from "aoconnect";

const { dryrun, message, results } = connect();

/*
  A manifest of active TFAO processes will be published once 
  they are live.  This value is just for demonstration purposes.
 */
const tfaoProcess = "P0BHJWnF4JjQ_NotARealTxId-qGsJdXa7yUDaqN3BOlYFYwAwR0"

/*
  Each of the files for the model you're using
  are the same as you would for Python Transformers or Pytorch, 
  but we reference them by their Arweave transaction id,
  since they must be stored on-chain.
  Example TX ids for the model and config data:
 */
const miniLM = {
  model: "ABCD",
  tokenizer: "EFGH",
  config: "IJKL"
}

const codeToRun = `
   wd = require("weavedrive")
   bert = require("transformers").bert
   
   local opts = {
     model = wd.getData("${miniLM.model}"),
     tokenizer = wd.getData("${miniLM.tokenizer}"),
     config = wd.getData("${miniLM.config}"),
     prompt = "Hello world."
   }
   
   return bert.encode_text(opts)
`;


/* 
  You could use dryrun to evaluate your code and run 
  your desired model, which does not save the results to memory.
 */
const tempResult = await dryrun({
  process: tfaoProcess,
  tags: [{name: 'Action', value: 'Eval'}],
  data: codeToRun,
  });

/*
  You could send a message, which will run your code and store the result, and
  then read the result.
 */
const messageId = await message({
    process: processId,
    tags: [{name: 'Action', value: 'Eval'}],
    signer: createDataItemSigner(wallet),
    data: codeToRun,
});

let { Messages, Spawns, Output, Error } = await result({
    message: messageId,
    process: processId,
});
console.log(JSON.stringify({ Messages, Spawns, Output, Error }));
```


## Disclaimer
Please note that this is an independent community project and is not affiliated with or endorsed by HuggingFace or AO.

#### License for Transformers
This work, like the original [Transformers library](https://github.com/huggingface/transformers)/[Candle library](https://github.com/huggingface/candle) by HuggingFace, is licensed under the Apache License 2.0. You can find the full text of the license in the [LICENSE](./LICENSE) file. Contributions to this project will also be made available under the same license.
  
#### License for AO
* **Licensed Work:** ao codebase. The Licensed Work is (c) 2024 Forward Research.
* **Additional Use Grant:** The ao codebase is offered under the BSL 1.1 license for the duration of the testnet period. After the testnet phase is over, the code will be made available under either a new evolutionary forking license or a traditional OSS license (GPLv3/v2, MIT, etc). For more information, please consult this [article on Arweave's medium](https://arweave.medium.com/arweave-is-an-evolutionary-protocol-e072f5e69eaa).
* **Change Date:** Four years from the date the Licensed Work is published.
* **Change License:** MPL 2.0.
  
For more information about the original AO library, please see the [Permaweb/ao GitHub](https://github.com/permaweb/ao).

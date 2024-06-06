use base64::prelude::{BASE64_STANDARD as b64, Engine};
use mlua::prelude::*;

use crate::models::common;
use candle_transformers::models::bert::{BertModel, Config, HiddenAct, DTYPE};
use candle_core::{DType, Device, Tensor, NdArray};
use candle_nn::VarBuilder;
use mlua::UserData;
// use rayon::ThreadPoolBuilder;
use tokenizers::{PaddingParams, Tokenizer};
use wasm_bindgen::prelude::*;


#[derive(serde::Serialize, serde::Deserialize)]
struct Embedding {
    data: Vec<f32>,
    prompt: String,
    model_id: String,
}

impl Embedding {
    fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }
}
impl UserData for Embedding {}
#[derive(Debug)]
struct Args {
    /// The model to use, check out available models: https://huggingface.co/models?library=sentence-transformers&sort=trending
    // model_id: Option<String>,
    model: Vec<u8>, //String,
    config: String,
    tokenizer: Vec<u8>, //String,
    // revision: Option<String>,
    /// When set, compute embeddings for this prompt.
    prompt: String,
    /// Use the pytorch weights rather than the safetensors ones
    // use_pth: bool,
    /// L2 normalization for embeddings. default_value = "true"
    normalize_embeddings: bool,
    /// Use tanh based approximation for Gelu instead of erf implementation. default_value = "false"
    approximate_gelu: bool,
    device: Device,
}

impl Args {
    fn build_model_and_tokenizer(&self) -> LuaResult<(BertModel, Tokenizer)> {
        let mut config: Config = serde_json::from_str::<Config>(&self.config)
            .map_err(|err| LuaError::external(err))?;
        if self.approximate_gelu {
            config.hidden_act = HiddenAct::GeluApproximate;
        }
        let model_bytes: Vec<u8> = self.model.clone(); //.as_bytes().to_vec();
        let vb = VarBuilder::from_buffered_safetensors(
            model_bytes,
            DTYPE,
            &self.device,
        ).map_err(|err| LuaError::external(err))?;
        let model = BertModel::load(vb, &config)
            .map_err(|err| LuaError::external(err))?;

        let tokenizer = Tokenizer::from_bytes(self.tokenizer.clone())//.as_bytes())
            .map_err(|err| LuaError::external(err))?;

        Ok((model, tokenizer))
    }
    fn _encode_text(&self) -> LuaResult<Vec<f32>> {
        eprintln!("Starting _encode_text");
        // let (model, mut tokenizer) = self.build_model_and_tokenizer()
        //     .map_err(|err| LuaError::external(err))?;
        ////////////////////////////////////////////////////////////////////////////////////////////
        let config = b64.decode(&self.config)
            .map_err(|err| {
                eprintln!("!! Error during b64 decode config\n{}", err);
                LuaError::external(err)
            })?;
        let mut config: Config = serde_json::from_slice(&*config)
            .map_err(|err| {
                eprintln!("!! Error during serde_json::from_value\n{}", err);
                LuaError::external(err)
            })?;
        eprintln!("Built mut config");
        if self.approximate_gelu {
            config.hidden_act = HiddenAct::GeluApproximate;
        }
        eprintln!("Maybe added approx gelu");

        // let model_bytes: Vec<u8> = self.model.clone();
        eprintln!("Let model_bytes");
        let vb = VarBuilder::from_buffered_safetensors(self.model.clone(), DTYPE,  &self.device)
            .map_err(|err| LuaError::external(err))?;
        eprintln!("Got VB");
        let model = BertModel::load(vb, &config)
            .map_err(|err| {
                eprintln!("!! Error model = BertModel:: ...\n{}", err);
                LuaError::external(err)
            })?;
        eprintln!("Did let model = BertModel::load(vb, &config)");

        let mut tokenizer = Tokenizer::from_bytes(self.tokenizer.clone())
            .map_err(|err| {
                eprintln!("!! Error during Tokenizer::from_bytes\n{}", err);
                LuaError::external(err)
            })?;
        eprintln!("Did let mut tokenizer = Tokenizer::from_bytes(s");

        ////////////////////////////////////////////////////////////////////////////////////////////
        eprintln!("Built model and tokenizer");
        let tokenizer = tokenizer
            .with_padding(None)
            .with_truncation(None)
            .map_err(|err| LuaError::external(err))?;
        let tokens = tokenizer.encode(&*self.prompt, true)
            .map_err(|err| LuaError::external(err))?
            .get_ids().to_vec();
        eprintln!("Got tokens");
        let token_ids = Tensor::new(&tokens[..], &self.device)
            .map_err(|err| {
                eprintln!("token_ids ... Tensor::new\n {}", err);
                LuaError::external(err)
            })?
            .unsqueeze(0)
            .map_err(|err| {
                eprintln!("token_ids ... unsqueeze\n {}", err);
                LuaError::external(err)
            })?;
        let token_type_ids = token_ids.zeros_like()
            .map_err(|err| {
                eprintln!("token_type_ids ... zeros_like\n {}", err);
                LuaError::external(err)
            })?;
        let embeddings = model.forward(&token_ids, &token_type_ids)
            .map_err(|err| {
                eprintln!("embeddings ... model.forward\n {}", err);
                LuaError::external(err)
            })?;

        // Apply some avg-pooling by taking the mean embedding value for all tokens (including padding)
        let (_n_sentence, n_tokens, _hidden_size) = embeddings.dims3()
            .map_err(|err| LuaError::external(err))?;
        let emb_sum = embeddings.sum(1)
            .map_err(|err| LuaError::external(err))?;
        let embeddings = (emb_sum / (n_tokens as f64))
            .map_err(|err| LuaError::external(err))?;
        let embeddings = if self.normalize_embeddings {
            common::normalize_l2(&embeddings).map_err(|err| LuaError::external(err))?
        } else {
            embeddings
        };
        let embeddings_data = embeddings
            .flatten_all()
            .map_err(|err| {
                eprintln!("!! Error during embeddings_data.flatten_all() ...\n{}", err);
                LuaError::external(err)
            })?
            .to_vec1()
            .map_err(|err| {
                eprintln!("!! Error during embeddings_data.to_vec1 ...\n{}", err);
                LuaError::external(err)
            })?;
        // let shp = embeddings_data.shape()
        //     .map_err(|err| LuaError::external(err))?;

        Ok(embeddings_data)
    }
}
// fn initialize_rayon() {
//     ThreadPoolBuilder::new().num_threads(1).build_global().unwrap();
// }
// fn encode_text(_lua: &Lua, table: LuaTable) -> LuaResult<Vec<f32>> {
fn encode_text(_lua: &Lua, table: LuaTable) -> LuaResult<String> {
    eprintln!("Starting encode_text");
    // TODO: remove the hard-coded model tokenizer config and get it from the passed table
    // let model: Option<String> = table.get("model")?;
    let model_id = "sentence-transformers/all-MiniLM-L6-v2";
    let model = Some(include_str!("data/sentence-transformers_all-MiniLM-L6-v2/model.safetensors.b64").to_string());
    eprintln!("Got model");
    let model = match model {
        Some(model) => b64.decode(model).map_err(|e| LuaError::external(e))?,
        None => unreachable!(),
    };
    eprintln!("Decoded model");
    // let config: Option<String> = table.get("config")?;
    // let config = Some(include_str!("data/sentence-transformers_all-MiniLM-L6-v2/config.json.b64").to_string());
    // let config = match config {
    //     Some(config) => {
    //         let dec = b64.decode(config)
    //                 .map_err(|e| LuaError::external(e))?;
    //         String::from_utf8(dec).map_err(|e| LuaError::external(e))?
    //     },
    //     None => unreachable!(),
    // };
    let config = include_str!("data/sentence-transformers_all-MiniLM-L6-v2/config.json.b64").to_string();

    eprintln!("Got config");
    // let tokenizer: String = table.get("tokenizer")?;
    let tokenizer = include_str!("data/sentence-transformers_all-MiniLM-L6-v2/tokenizer.json.b64");
    let tokenizer = b64.decode(tokenizer)
        .map_err(|e| LuaError::external(e))?;
    eprintln!("Got tokenizer");

    // print("Bert args parsed");
    let args = Args {
        model, config, tokenizer,
        prompt: table.get("prompt").unwrap_or("The cat sits outside".to_string()), // When set, compute embeddings for this prompt.
        // use_safetensors: !table.get("use_safetensors").unwrap_or(true), // Use the pytorch weights rather than the safetensors ones
        normalize_embeddings: table.get("normalize_embeddings").unwrap_or(true), // L2 normalization for embeddings. default_value = "true"
        approximate_gelu: table.get("approximate_gelu").unwrap_or(false), // Use tanh based approximation for Gelu instead of erf implementation. default_value = "false"
        device: Device::Cpu,
    };
    eprintln!("Prepped Args");
    eprintln!("Prompt provided is: {}", args.prompt);

    // let embeddings = _encode_text(args).map_err(|err| LuaError::external(err))?;
    let embeddings = args._encode_text()
        .map_err(|err| {
            eprintln!("Error in encode_text when called _encode_text()\n{}", err);
            LuaError::external(err)
        })?;
    // let embeddings: String = serde_json::to_string(&embeddings).map_err(|err| LuaError::external(err))?;
    // println!("{}", embeddings);
    // let embeddings = Embeddings {
    //     data: embeddings,
    // };
    let output = Embedding {
        data: embeddings,
        prompt: args.prompt,
        model_id: model_id.to_string(),
    };
    let output_str = output.to_json()
        .map_err(|err| {
            eprintln!("Error in serializing embeddings\n{}", err);
            LuaError::external(err)
        })?;
    eprintln!("{}", output_str);
    Ok(output_str)
}

// #[mlua::lua_module]
pub fn preload(lua: &Lua) -> LuaResult<()> {
    let package: LuaTable = lua.globals().get("package")?;
    let loaded: LuaTable = package.get("loaded")?;
    let bert_module_table = lua.create_table()?;
    // let config_table = lua.create_table()?;
    // config_table.set("model", include_str!("data/all-mpnet-base-v2/model.safetensors.b64"))?;
    // config_table.set("tokenizer", include_str!("data/all-mpnet-base-v2/tokenizer.json.b64"))?;
    // config_table.set("config", include_str!("data/all-mpnet-base-v2/config.json.b64"))?;
    // bert_module_table.set("config", config_table)?;


    // let _encode_text_func: LuaFunction = lua.create_function(|_, t: LuaTable| {
    //     let json_str = serde_json::to_string(&t).map_err(LuaError::external)?;
    //     Ok(json_str)
    // })?;
    // let lua_encode_text_func = lua.create_thread(lua.create_function(encode_text)?)?;
    let lua_encode_text_func = lua.create_function(encode_text)?;
    bert_module_table.set("encode_text", lua_encode_text_func)?;
    loaded.set("bert", bert_module_table)?;
    Ok(())
}

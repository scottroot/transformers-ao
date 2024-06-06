#![allow(unused)]
use std::io::Write;
use std::path::PathBuf;

use mlua::prelude::*;
use crate::common::{normalize_l2};

use candle_transformers::models::t5;

use anyhow::{Error as AnyError, Result as AnyResult};
use candle_core::{DType, Device, Tensor};
use candle_nn::VarBuilder;
use candle_transformers::generation::LogitsProcessor;
use mlua::{Table, UserData};
use tokenizers::Tokenizer;


// Which::T5Base => ("t5-base", "main"),
// Which::T5Small => ("t5-small", "refs/pr/15"),
// Which::T5Large => ("t5-large", "main"),
// Which::T5_3B => ("t5-3b", "main"),
// Which::Mt5Base => ("google/mt5-base", "refs/pr/5"),
// Which::Mt5Small => ("google/mt5-small", "refs/pr/6"),
// Which::Mt5Large => ("google/mt5-large", "refs/pr/2"),
//
// weights_filename =
// if model_id == "google/flan-t5-xxl" || model_id == "google/flan-ul2" {
//         hub_load_safetensors(&repo, "model.safetensors.index.json")?
//     } else {
//         vec![repo.get("model.safetensors")?]
//     }
// }

// tokenizer_filename = match &args.tokenizer_file {
// None => match args.which.unwrap_or(Which::T5Small) {
// Which::Mt5Base => api
//     .model("lmz/mt5-tokenizers".into())
//     .get("mt5-base.tokenizer.json")?,
// Which::Mt5Small => api
//     .model("lmz/mt5-tokenizers".into())
//     .get("mt5-small.tokenizer.json")?,
// Which::Mt5Large => api
//     .model("lmz/mt5-tokenizers".into())
//     .get("mt5-large.tokenizer.json")?,
// _ => repo.get("tokenizer.json")?,

const DTYPE: DType = DType::F32;

#[derive(Clone, Debug, Copy)]
enum Which {
    T5Base,
    T5Small,
    T5Large,
    T5_3B,
    Mt5Base,
    Mt5Small,
    Mt5Large,
}

#[derive(Debug, Clone)]
struct Args {
    device: Device,
    model: String,
    config: String,
    tokenizer: String,
    /// Enable decoding.
    decode: bool,
    /// Use this prompt, otherwise compute sentence similarities.
    prompt: String,
    /// If set along with --decode, will use this prompt to initialize the decoder.
    decoder_prompt: Option<String>,
    /// L2 normalization for embeddings. default_value = "true"
    normalize_embeddings: bool,
    /// The temperature used to generate samples. default_value_t = 0.8
    temperature: f64,
    /// Nucleus sampling probability cutoff.
    top_p: Option<f64>,
    /// Penalty to be applied for repeating tokens, 1. means no penalty. default_value_t = 1.1
    repeat_penalty: f32,
    /// The context size to consider for the repeat penalty. default_value_t = 64
    repeat_last_n: usize,
}

impl UserData for Args { }

struct T5ModelBuilder {
    model_bytes: Vec<u8>,
    config: t5::Config,
    device: Device,
}

impl T5ModelBuilder {
    pub fn load(args: &Args) -> AnyResult<(Self, Tokenizer)> {
        let device = Device::Cpu;
        let mut config: t5::Config = serde_json::from_str::<t5::Config>(&*args.config)
            .map_err(|err| LuaError::external(err))?;
        let model_bytes: Vec<u8> = args.model.clone().as_bytes().to_vec();
        let mut tokenizer = Tokenizer::from_bytes(args.tokenizer.as_bytes())
            .map_err(|err| LuaError::external(err))?;
        Ok((
            Self {
                model_bytes,
                config,
                device
            },
            tokenizer,
        ))
    }

    pub fn build_encoder(&self) -> AnyResult<t5::T5EncoderModel> {
        let vb = unsafe {
            VarBuilder::from_buffered_safetensors(self.model_bytes.clone(), DTYPE, &self.device)
                .map_err(|err| LuaError::external(err))?
        };
        let model: t5::T5EncoderModel = t5::T5EncoderModel::load(vb, &self.config)
            .map_err(|err| LuaError::external(err))?;
        Ok(model)
    }

    pub fn build_conditional_generation(&self) -> AnyResult<t5::T5ForConditionalGeneration> {
        let vb = unsafe {
            VarBuilder::from_buffered_safetensors(self.model_bytes.clone(), DTYPE, &Device::Cpu)
                .map_err(|err| LuaError::external(err))?
        };
        let model: t5::T5ForConditionalGeneration = t5::T5ForConditionalGeneration::load(vb, &self.config)
            .map_err(|err| LuaError::external(err))?;
        Ok(model)
    }
}

fn __t5(args: Args) -> LuaResult<()> {
    let (builder, mut tokenizer) = T5ModelBuilder::load(&args).map_err(LuaError::external)?;
    let device = &builder.device;
    let tokenizer = tokenizer.with_padding(None)
        .with_truncation(None)
        .map_err(LuaError::external)?;
    let prompt = args.prompt;
    let tokens = tokenizer
        .encode(prompt, true)
        .map_err(LuaError::external)?
        .get_ids()
        .to_vec();
    let input_token_ids = Tensor::new(&tokens[..], device)
        .map_err(LuaError::external)?
        .unsqueeze(0)
        .map_err(LuaError::external)?;
    if !args.decode {
        let mut model = builder
            .build_encoder()
            .map_err(LuaError::external)?;
        let start = std::time::Instant::now();
        let embedding = model
            .forward(&input_token_ids)
            .map_err(LuaError::external)?;
        println!("{embedding}");
        println!("Took {:?}", start.elapsed());
    } else {
        let mut model = builder
            .build_conditional_generation()
            .map_err(LuaError::external)?;
        let mut output_token_ids = [builder
            .config
            .decoder_start_token_id
            .unwrap_or(builder.config.pad_token_id)
            as u32
        ].to_vec();
        if let Some(decoder_prompt) = &args.decoder_prompt {
            print!("{decoder_prompt}");
            output_token_ids.extend(
                tokenizer
                    .encode(decoder_prompt.to_string(), false)
                    .map_err(LuaError::external)?
                    .get_ids()
                    .to_vec(),
            );
        }
        let temperature = if args.temperature <= 0. {
            None
        } else {
            Some(args.temperature)
        };
        let mut logits_processor = LogitsProcessor::new(
            299792458,
            temperature,
            args.top_p
        );
        let encoder_output = model.encode(&input_token_ids)
            .map_err(LuaError::external)?;
        let start = std::time::Instant::now();

        for index in 0.. {
            if output_token_ids.len() > 512 {
                break;
            }
            let decoder_token_ids = if index == 0 || !builder.config.use_cache {
                Tensor::new(output_token_ids.as_slice(), device)
                    .map_err(LuaError::external)?
                    .unsqueeze(0)
                    .map_err(LuaError::external)?
            } else {
                let last_token = *output_token_ids.last().unwrap();
                Tensor::new(&[last_token], device)
                    .map_err(LuaError::external)?
                    .unsqueeze(0)
                    .map_err(LuaError::external)?
            };
            let logits = model
                .decode(&decoder_token_ids, &encoder_output)
                .map_err(LuaError::external)?
                .squeeze(0)
                .map_err(LuaError::external)?;
            let logits = if args.repeat_penalty == 1. {
                logits
            } else {
                let start_at = output_token_ids.len().saturating_sub(args.repeat_last_n);
                candle_transformers::utils::apply_repeat_penalty(
                    &logits,
                    args.repeat_penalty,
                    &output_token_ids[start_at..],
                ).map_err(LuaError::external)?
            };

            let next_token_id = logits_processor.sample(&logits).map_err(LuaError::external)?;
            if next_token_id as usize == builder.config.eos_token_id {
                break;
            }
            output_token_ids.push(next_token_id);
            if let Some(text) = tokenizer.id_to_token(next_token_id) {
                let text = text
                    .replace('▁', " ")
                    .replace("<0x0A>", "\n");
                print!("{text}");
                std::io::stdout().flush().map_err(LuaError::external)?;
            }
        }
        let dt = start.elapsed();
        println!(
            "\n{} tokens generated ({:.2} token/s)\n",
            output_token_ids.len(),
            output_token_ids.len() as f64 / dt.as_secs_f64(),
        );
    }
    Ok(())
}

pub fn main(lua: &Lua, table: Table) -> LuaResult<()> {
    let args = Args {
        model: table.get("model")?,
        config: table.get("config")?,
        tokenizer: table.get("tokenizer")?,
        device: Device::Cpu,
        /// Enable decoding.
        decode: table.get("decode").unwrap_or(true),
        /// Use this prompt, otherwise compute sentence similarities.
        // prompt: "Do cats eat fruit from trees that has fallen to the ground where they can reach it?".to_string(),
        prompt: table.get("prompt")?,
        /// If set along with --decode, will use this prompt to initialize the decoder.
        // decoder_prompt: Option::from("Answer this question in English: ".to_string()), // Option<String>,
        decoder_prompt: table.get("decoder_prompt")?,
        /// L2 normalization for embeddings. default_value = "true"
        normalize_embeddings: true,
        /// The temperature used to generate samples. default_value_t = 0.8
        temperature: table.get("temperature").unwrap_or(0.8f64),
        /// Nucleus sampling probability cutoff.
        top_p: table.get("top_p")?, // Option<f64>,
        /// Penalty to be applied for repeating tokens, 1. means no penalty. default_value_t = 1.1
        repeat_penalty: table.get("repeat_penalty").unwrap_or(1.1f32),
        /// The context size to consider for the repeat penalty. default_value_t = 64
        repeat_last_n: table.get("repeat_last_n").unwrap_or(64usize),
    };

    let embeddings = __t5(args).map_err(|err| LuaError::external(err))?;
    Ok(())
}

// #[mlua::lua_module]
// fn t5(lua: &Lua) -> LuaResult<LuaTable> {
//     let exports = lua.create_table()?;
//     exports.set("t5", lua.create_function(_t5)?)?;
//     Ok(exports)
// }

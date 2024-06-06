#![allow(unused)]

use mlua::prelude::*;
use mlua::UserData;
use wasm_bindgen::prelude::*;
use std::error::Error;
use std::io::Cursor;
use std::sync::Arc;
use std::thread;
use candle_core::{DType, Device, IndexOp, Module, Tensor, D};

use tokenizers::Tokenizer;
use anyhow::{Error as AnyError, Result as AnyResult};
use base64::engine::general_purpose::STANDARD;
use candle_core::safetensors::BufferedSafetensors;
use candle_nn::VarBuilder;
// use candle_transformers::models::stable_diffusion;
use candle_transformers::models::stable_diffusion::{
    // StableDiffusionConfig,
    clip,
    unet_2d::{UNet2DConditionModel, UNet2DConditionModelConfig},
    vae::{AutoEncoderKL, AutoEncoderKLConfig}
};
use base64::prelude::*;
use image::codecs::png::PngEncoder;
use image::{ColorType, ExtendedColorType, ImageEncoder};
use crate::common::{image_preprocess, save_image_file};
use crate::stable_diffusion_config::StableDiffusionConfig;


struct Args {
    /// The prompt to be used for image generation.
    prompt: String,
    uncond_prompt: String,  // default_value = ""
    /// The height in pixels of the generated image.
    height: usize,
    /// The width in pixels of the generated image.
    width: usize,
    /// The CLIP weight file, in .safetensors format passed in as string.
    clip_weights: Vec<u8>,
    /// The VAE weight file, in .safetensors format.
    vae_weights: Vec<u8>,
    /// The UNet weight file, in .safetensors format.
    unet_weights: Vec<u8>,
    /// The file specifying the tokenizer to use for tokenization.
    tokenizer: Tokenizer,
    /// The size of the sliced attention or 0 for automatic slicing (disabled by default)
    sliced_attention_size: Option<usize>,
    /// The number of steps to run the diffusion for.
    n_steps: usize,
    /// The number of samples to generate iteratively.
    num_samples: usize,
    /// The numbers of samples to generate simultaneously.
    bsize: Option<usize>,
    /// The name of the final image to generate.
    final_image: String,
    sd_version: StableDiffusionVersion,
    /// Generate intermediary images at each step.
    intermediary_images: bool,
    use_flash_attn: bool,
    use_f16: bool,
    guidance_scale: f64,
    img2img: Option<String>,
    /// The strength, indicates how much to transform the initial image. The
    /// value must be between 0 and 1, a value of 1 discards the initial image
    /// information.
    img2img_strength: f64,
    /// The seed to use when generating random samples.
    seed: u64,
    first: bool,
    dtype: DType,
    device: Device,
    sd_config: StableDiffusionConfig,
}

impl UserData for Args { }

impl<'lua> FromLua<'lua> for Args {
    fn from_lua(value: LuaValue<'lua>, _: &'lua Lua) -> LuaResult<Self> {
        match value {
            LuaValue::Table(table) => {
                let prompt: String = table.get("prompt")?;
                let uncond_prompt: String = table.get("uncond_prompt").unwrap_or(String::from(""));
                let height: usize = table.get("height").unwrap_or(1024);
                let width: usize = table.get("width").unwrap_or(768);

                let clip_weights: String = table.get("clip_model")?;
                let clip_weights = BASE64_STANDARD.decode(clip_weights)
                    .map_err(|err| LuaError::RuntimeError(err.to_string()))?;
                // let clip_weights: Vec<u8> = clip_weights.as_bytes().to_vec();

                let vae_weights: String = table.get("vae_model")?;
                let vae_weights: Vec<u8> = BASE64_STANDARD.decode(vae_weights)
                    .map_err(|err| LuaError::RuntimeError(err.to_string()))?;
                // let vae_weights: Vec<u8> = vae_weights.as_bytes().to_vec();

                let unet_weights: String = table.get("unet_model")?;
                let unet_weights: Vec<u8> = BASE64_STANDARD.decode(unet_weights)
                    .map_err(|err| LuaError::RuntimeError(err.to_string()))?;
                // let unet_weights: Vec<u8> = unet_weights.as_bytes().to_vec();

                let tokenizer: String = table.get("tokenizer")?;
                // let tokenizer: Vec<u8> = tokenizer.as_bytes().to_vec();
                let tokenizer: Vec<u8> = BASE64_STANDARD.decode(tokenizer)
                    .map_err(|err| LuaError::RuntimeError(err.to_string()))?;
                let mut tokenizer = Tokenizer::from_bytes(tokenizer)
                    .map_err(|err| LuaError::RuntimeError(err.to_string()))?;

                let sd_version: String = table.get("sd_version")
                    .unwrap_or(String::from("v1_5"))
                    .replace(".", "_")
                    .to_lowercase();
                let sd_version: StableDiffusionVersion = match sd_version.as_str() {
                    "v1_5" => Ok(StableDiffusionVersion::V1_5),
                    "v2_1" => Ok(StableDiffusionVersion::V2_1),
                    "xl" => Ok(StableDiffusionVersion::Xl),
                    "turbo" => Ok(StableDiffusionVersion::Turbo),
                    _ => Err(mlua::Error::FromLuaConversionError {
                        from: "string",
                        to: "StableDiffusionVersion",
                        message: Some(format!("invalid stable diffusion version: {}", sd_version)),
                    }),
                }?;
                let n_steps: Option<usize> = table.get("n_steps")?;
                let n_steps = match n_steps {
                    Some(n_steps) => n_steps,
                    None => match sd_version {
                        StableDiffusionVersion::V1_5
                        | StableDiffusionVersion::V2_1
                        | StableDiffusionVersion::Xl => 30,
                        StableDiffusionVersion::Turbo => 1,
                    },
                };
                let num_samples: usize = table.get("num_samples").unwrap_or(1);
                let bsize: Option<usize> = table.get("bsize")?;
                let sliced_attention_size: Option<usize> = table.get("sliced_attention_size").unwrap_or(Some(0));
                let final_image: String = table.get("final_image").unwrap_or(String::from("sd_final.png"));

                let intermediary_images: bool = table.get("intermediary_images")?;
                let use_flash_attn: bool = table.get("use_flash_attn")?;
                let guidance_scale: Option<f64> = table.get("guidance_scale")?;
                let guidance_scale: f64 = match guidance_scale {
                    Some(guidance_scale) => guidance_scale,
                    None => match sd_version {
                        StableDiffusionVersion::V1_5
                        | StableDiffusionVersion::V2_1
                        | StableDiffusionVersion::Xl => 7.5,
                        StableDiffusionVersion::Turbo => 0.,
                    },
                };
                let img2img: Option<String> = table.get("img2img")?;
                let img2img_strength: f64 = table.get("img2img_strength").unwrap_or(0.8);
                if !(0. ..=1.).contains(&img2img_strength) {
                    return Err(LuaError::RuntimeError(
                        format!(
                            "Stable-Diffusion arg 'img2img_strength' should be between 0 and 1, got {}",
                            img2img_strength
                        )
                    ))
                }
                let first: bool = table.get("first")?;
                let device: Device = Device::Cpu;
                let use_f16: bool = table.get("use_f16").unwrap_or(false);
                let dtype: DType = if use_f16 { DType::F16 } else { DType::F32 };
                let seed: u64 = table.get("seed").unwrap_or(42u64);
                device.set_seed(seed)
                    .map_err(|e| LuaError::ExternalError);

                let sd_config = match sd_version {
                    StableDiffusionVersion::V1_5 => {
                        StableDiffusionConfig::v1_5(sliced_attention_size, Some(height), Some(width))
                    }
                    StableDiffusionVersion::V2_1 => {
                        StableDiffusionConfig::v2_1(sliced_attention_size, Some(height), Some(width))
                    }
                    StableDiffusionVersion::Xl => {
                        StableDiffusionConfig::sdxl(sliced_attention_size, Some(height), Some(width))
                    }
                    StableDiffusionVersion::Turbo => {
                        StableDiffusionConfig::sdxl_turbo(sliced_attention_size, Some(height), Some(width))
                    }
                };
                Ok(Args {
                    prompt, uncond_prompt, width, height, clip_weights, unet_weights, vae_weights,
                    tokenizer, sliced_attention_size, n_steps, num_samples, bsize, final_image,
                    sd_version, intermediary_images, use_flash_attn, use_f16, guidance_scale,
                    img2img, img2img_strength, seed, first, dtype, device, sd_config
                })
            },
            _ => {
                Err(mlua::Error::FromLuaConversionError {
                    from: value.type_name(),
                    to: "Args",
                    message: None,
                })
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StableDiffusionVersion {
    V1_5,
    V2_1,
    Xl,
    Turbo,
}



#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ModelFile {
    Tokenizer,
    Tokenizer2,
    Clip,
    Clip2,
    Unet,
    Vae,
}

fn output_filename(
    basename: &str,
    sample_idx: usize,
    num_samples: usize,
    timestep_idx: Option<usize>,
) -> String {
    let filename = if num_samples > 1 {
        match basename.rsplit_once('.') {
            None => format!("{basename}.{sample_idx}.png"),
            Some((filename_no_extension, extension)) => {
                format!("{filename_no_extension}.{sample_idx}.{extension}")
            }
        }
    } else {
        basename.to_string()
    };
    match timestep_idx {
        None => filename,
        Some(timestep_idx) => match filename.rsplit_once('.') {
            None => format!("{filename}-{timestep_idx}.png"),
            Some((filename_no_extension, extension)) => {
                format!("{filename_no_extension}-{timestep_idx}.{extension}")
            }
        },
    }
}

// #[allow(clippy::too_many_arguments)]
// fn save_image(
//     vae: &AutoEncoderKL,
//     latents: &Tensor,
//     vae_scale: f64,
//     bsize: usize,
//     idx: usize,
//     final_image: &str,
//     num_samples: usize,
//     timestep_ids: Option<usize>,
// ) -> Result<(), candle_core::Error> {
//     let images = vae.decode(&(latents / vae_scale)?)?;
//     let images = ((images / 2.)? + 0.5)?.to_device(&Device::Cpu)?;
//     let images = (images.clamp(0f32, 1.)? * 255.)?.to_dtype(DType::U8)?;
//     for batch in 0..bsize {
//         let image = images.i(batch)?;
//         let image_filename = output_filename(
//             final_image,
//             (bsize * idx) + batch + 1,
//             batch + num_samples,
//             timestep_ids,
//         );
//         save_image_file(&image, image_filename)?;
//     }
//     println!("Finished batch");
//     Ok(())
// }
fn generate_image(
    vae: &AutoEncoderKL,
    latents: &Tensor,
    vae_scale: f64,
    bsize: usize,
    idx: usize,
    num_samples: usize,
    timestep_ids: Option<usize>,
) -> Result<String, candle_core::Error> {
    let images = vae.decode(&(latents / vae_scale)?)?;
    let images = ((images / 2.)? + 0.5)?.to_device(&Device::Cpu)?;
    let images = (images.clamp(0f32, 1.)? * 255.)?.to_dtype(DType::U8)?;
    // let mut image_buffers = Vec::new();
    // for batch in 0..bsize {
    //     let image = images.i(batch)?;
    //     let image = image.permute((1, 2, 0))?.flatten_all()?;
    //     let pixels = image.to_vec1::<u8>()?;
    //     let (height, width) = image.dims2()?;
    //     let image: image::ImageBuffer<image::Rgb<u8>, Vec<u8>> =
    //         match image::ImageBuffer::from_raw(width as u32, height as u32, pixels) {
    //             Some(image) => image,
    //             None => candle_core::bail!("Error while creating image buffer"),
    //         };
    //
    //     let mut buffer = Cursor::new(Vec::new());
    //     let encoder = PngEncoder::new(&mut buffer);
    //     match encoder.write_image(&image, image.width(), image.height(), ExtendedColorType::from(ColorType::Rgb8)) {
    //        Ok(()) => {},
    //        Err(e) => candle_core::bail!("Error while encoding image buffer to PNG: {}", e),
    //     };
    //
    //     image_buffers.push(buffer.into_inner());
    // }
    // let images = vae.decode(&(latents / vae_scale)?)?;
    // let images = ((images / 2.)? + 0.5)?.to_device(&Device::Cpu)?;
    // let images = (images.clamp(0f32, 1.)? * 255.)?.to_dtype(DType::U8)?;
    let image = images.i(0)?;
    let (channel, height, width) = image.dims3()?;
    if channel != 3 {
        candle_core::bail!("save_image expects an input of shape (3, height, width)")
    }
    let image = image.permute((1, 2, 0))?.flatten_all()?;
    let pixels = image.to_vec1::<u8>()?;
    // let (height, width) = image.dims2()?;

    let image: image::ImageBuffer<image::Rgb<u8>, Vec<u8>> =
        image::ImageBuffer::from_raw(width as u32, height as u32, pixels)
            .ok_or_else(|| {
                println!("Error while creating image buffer");
                candle_core::Error::Msg("Error while creating image buffer".parse().unwrap())
            })?;

    let mut buffer = Cursor::new(Vec::new());
    let encoder = PngEncoder::new(&mut buffer);
    encoder.write_image(&image, image.width(), image.height(), ExtendedColorType::Rgb8)
        .map_err(|e| AnyError::msg(e));

    // Ok(buffer.into_inner())

    let image_b64 = BASE64_STANDARD.encode(buffer.into_inner());
    Ok(format!("data:image/png;base64,{}", image_b64))
}

#[allow(clippy::too_many_arguments)]
fn text_embeddings(
    lua: &Lua,
    prompt: &str,
    uncond_prompt: &str,
    tokenizer: Tokenizer,
    clip_weights: Vec<u8>,
    sd_version: StableDiffusionVersion,
    sd_config: &StableDiffusionConfig,
    use_f16: bool,
    device: &Device,
    dtype: DType,
    use_guide_scale: bool,
    first: bool,
) -> AnyResult<Tensor> {
    let globals = lua.globals();
    let _print: LuaFunction = globals.get("print")?;
    let print = |s: &str| { _print.call::<_, ()>(String::from(s)); };

    print("Starting text_embeddings first line of function");
    // let clip_config = if first {
    //     &sd_config.clip
    // } else {
    //     sd_config.clip2.as_ref().unwrap()
    // };
    // print("Set clip_config");
    //
    // let clip_vb = VarBuilder::from_buffered_safetensors(clip_weights, dtype, &device)
    //         .map_err(|err| LuaError::external(err))?;
    //
    // print("Set clip VarBuilder");
    // let text_model = clip::ClipTextTransformer::new(clip_vb, clip_config)
    //     .map_err(|err| {
    //         println!("{}", err);
    //         LuaError::external(err)
    //     })?;
    let text_model = sd_config.build_clip(clip_weights, device, dtype, first)
        .map_err(|err| {
            println!("{}", err);
            LuaError::external(err)
        })?;
    print("Created textmodel");
    let pad_id = match &sd_config.clip.pad_with {
        Some(padding) => *tokenizer.get_vocab(true).get(padding.as_str()).unwrap(),
        None => *tokenizer.get_vocab(true).get("<|endoftext|>").unwrap(),
    };
    println!("Running with prompt \"{prompt}\".");
    let mut tokens = tokenizer
        .encode(prompt, true)
        .map_err(AnyError::msg)?
        .get_ids()
        .to_vec();
    if tokens.len() > sd_config.clip.max_position_embeddings {
        anyhow::bail!(
            "the prompt is too long, {} > max-tokens ({})",
            tokens.len(),
            sd_config.clip.max_position_embeddings
        )
    }
    while tokens.len() < sd_config.clip.max_position_embeddings {
        tokens.push(pad_id)
    }
    let tokens = Tensor::new(tokens.as_slice(), &device)?.unsqueeze(0)?;

    println!("Building the Clip transformer.");

    let text_embeddings = text_model.forward(&tokens)
            .map_err(|err| LuaError::external(err))?;

    let text_embeddings = if use_guide_scale {
        let mut uncond_tokens = tokenizer
            .encode(uncond_prompt, true)
            .map_err(|err| LuaError::external(err))?
            .get_ids()
            .to_vec()
            ;
        if uncond_tokens.len() > sd_config.clip.max_position_embeddings {
            anyhow::bail!(
                "the negative prompt is too long, {} > max-tokens ({})",
                uncond_tokens.len(),
                sd_config.clip.max_position_embeddings
            )
        }
        while uncond_tokens.len() < sd_config.clip.max_position_embeddings {
            uncond_tokens.push(pad_id)
        }

        let uncond_tokens = Tensor::new(uncond_tokens.as_slice(), &device)
            .map_err(|err| LuaError::external(err))?
            .unsqueeze(0)
            .map_err(|err| LuaError::external(err))?
            ;
        let uncond_embeddings = text_model.forward(&uncond_tokens)
            .map_err(|err| LuaError::external(err))?;

        Tensor::cat(&[uncond_embeddings, text_embeddings], 0)
            .map_err(|err| LuaError::external(err))?
            .to_dtype(dtype)
            .map_err(|err| LuaError::external(err))?
    } else {
        text_embeddings.to_dtype(dtype).map_err(|err| LuaError::external(err))?
    };
    Ok(text_embeddings)
}


// fn run(lua: &Lua, args: Args) -> LuaResult<()> {
fn run(lua: &Lua, args: Args) -> LuaResult<mlua::Variadic<String>> {
    let globals = lua.globals();
    let _print: LuaFunction = globals.get("print")?;
    let print = |s: &str| { _print.call::<_, ()>(String::from(s)); };
    // let prompt = args.prompt.unwrap_or("".parse().unwrap());
    // let uncond_prompt = args.uncond_prompt.clone().unwrap_or(String::from(""));

    print("Starting to run, validating args...");

    let height = args.height;
    let width = args.width;
    let sliced_attention_size = args.sliced_attention_size;
    let sd_version = args.sd_version;
    let bsize = args.bsize.unwrap_or(1);
    let use_f16 = args.use_f16;

    let use_guide_scale = args.guidance_scale > 1.0;

    // print("About to create models...");
    // let (
    //     // text_model,
    //     vae_model,
    //     unet_model,
    //     tokenizer
    // ) = args
    //     .load_model(args.sd_config.clone())
    //     .map_err(|err| LuaError::external(err))?;
    // print("Created models...");

    let scheduler = args.sd_config
        .build_scheduler(args.n_steps)
        .map_err(|e| LuaError::RuntimeError(e.to_string()))?;

    let which = match sd_version {
        StableDiffusionVersion::Xl | StableDiffusionVersion::Turbo => vec![true, false],
        _ => vec![true],
    };

    // print("prepare tokenizer");
    // let mut tokenizer = Tokenizer::from_bytes(args.tokenizer.clone().as_bytes())
    //         .map_err(|err| LuaError::external(err))?;

    // let clip_model_bytes: Vec<u8> = args.clip_weights.as_bytes().to_vec();
    print("About to start text_embeddings");
    let text_embeddings = which
        .iter()
        .map(|first| {
            text_embeddings(
                &lua,
                &args.prompt,
                &args.uncond_prompt,
                args.tokenizer.clone(),
                args.clip_weights.clone(),
                sd_version,
                &args.sd_config,
                args.use_f16,
                &args.device,
                args.dtype,
                use_guide_scale,
                *first,
            )
        })
        .collect::<Result<Vec<_>, AnyError>>() // ?;
        // .map_err(|e| LuaError::RuntimeError(e.to_string()))?;
        .map_err(|e| {
            println!("Error: {:?}", e);
            LuaError::external(e)
        })?;

    print("Line 477");

    let text_embeddings = Tensor::cat(&text_embeddings, D::Minus1)
        .map_err(|e| LuaError::RuntimeError(e.to_string()))?;

    let text_embeddings = text_embeddings
        .repeat((bsize, 1, 1))
        // .map_err(|e| LuaError::ExternalError);
        .map_err(|e| LuaError::RuntimeError(e.to_string()))?;
    print("finished text embeddings");

    // Create VAE Model
    // https://huggingface.co/runwayml/stable-diffusion-v1-5/blob/main/vae/config.json
    let vae_model: AutoEncoderKL = args.sd_config.build_vae(
        args.vae_weights,
        &args.device,
        args.dtype,
    ).map_err(|e| LuaError::RuntimeError(e.to_string()))?;

    print("Finished vae_model");
    // Create UNET Model
    let unet_model = args.sd_config.build_unet(
        args.unet_weights.clone(),
        &args.device,
        4,
        args.use_flash_attn,
        args.dtype
    ).map_err(|e| {
        print(&*e.to_string());
        println!("{}", e.to_string());
        LuaError::RuntimeError(e.to_string())
    })?;

    let init_latent_dist = match &args.img2img {
        None => None,
        Some(image) => {
            let image = image_preprocess(image)
                .map_err(|e| LuaError::RuntimeError(e.to_string()))?
                .to_device(&args.device)
                .map_err(|e| LuaError::RuntimeError(e.to_string()))?;
            Some(vae_model
                .encode(&image)
                .map_err(|e| LuaError::RuntimeError(e.to_string()))?
            )
        }
    };

    let t_start = if args.img2img.is_some() {
        args.n_steps - (args.n_steps as f64 * args.img2img_strength) as usize
    } else {
        0
    };

    let vae_scale = match sd_version {
        StableDiffusionVersion::V1_5
        | StableDiffusionVersion::V2_1
        | StableDiffusionVersion::Xl => 0.18215,
        StableDiffusionVersion::Turbo => 0.13025,
    };

    let mut images_array = Vec::new();

    for idx in 0..args.num_samples {
        println!("Starting batch {}", idx);
        let timesteps = scheduler.timesteps();
        let latents = match &init_latent_dist {
            Some(init_latent_dist) => {
                let latents = init_latent_dist.sample()
                    .map_err(|e| LuaError::RuntimeError(e.to_string()))?;
                let latents = (latents * vae_scale)
                    .map_err(|e| LuaError::RuntimeError(e.to_string()))?
                    .to_device(&args.device)
                    .map_err(|e| LuaError::RuntimeError(e.to_string()))?;
                if t_start < timesteps.len() {
                    let noise = latents
                        .randn_like(0f64, 1f64)
                        .map_err(|e| LuaError::RuntimeError(e.to_string()))?;
                    scheduler
                        .add_noise(&latents, noise, timesteps[t_start])
                        .map_err(|e| LuaError::RuntimeError(e.to_string()))?
                } else {
                    latents
                }
            }
            None => {
                let latents = Tensor::randn(
                    0f32,
                    1f32,
                    (bsize, 4, args.sd_config.height / 8, args.sd_config.width / 8),
                    &args.device,
                ).map_err(|e| LuaError::RuntimeError(e.to_string()))?;
                // scale the initial noise by the standard deviation required by the scheduler
                (latents * scheduler.init_noise_sigma())
                    .map_err(|e| LuaError::RuntimeError(e.to_string()))?
            }
        };
        let mut latents = latents
            .to_dtype(args.dtype)
            .map_err(|e| LuaError::RuntimeError(e.to_string()))?;

        println!("starting sampling");
        for (timestep_index, &timestep) in timesteps.iter().enumerate() {
            if timestep_index < t_start {
                continue;
            }
            let start_time = std::time::Instant::now();
            let latent_model_input = if use_guide_scale {
                Tensor::cat(&[&latents, &latents], 0)
                    .map_err(|e| LuaError::RuntimeError(e.to_string()))?
            } else {
                latents.clone()
            };

            let latent_model_input = scheduler
                .scale_model_input(latent_model_input, timestep)
                .map_err(|e| LuaError::RuntimeError(e.to_string()))?;
            let noise_pred = unet_model
                .forward(&latent_model_input, timestep as f64, &text_embeddings)
                .map_err(|e| LuaError::RuntimeError(e.to_string()))?;

            let noise_pred = if use_guide_scale {
                let noise_pred = noise_pred
                    .chunk(2, 0)
                    .map_err(|e| LuaError::RuntimeError(e.to_string()))?;

                let (noise_pred_uncond, noise_pred_text) = (&noise_pred[0], &noise_pred[1]);
                (
                    noise_pred_uncond + (
                        (noise_pred_text - noise_pred_uncond).map_err(|e| LuaError::RuntimeError(e.to_string()))?
                            * args.guidance_scale
                    ).map_err(|e| LuaError::RuntimeError(e.to_string()))?
                ).map_err(|e| LuaError::RuntimeError(e.to_string()))?
            } else {
                noise_pred
            };

            latents = scheduler.step(&noise_pred, timestep, &latents)
                .map_err(|e| LuaError::RuntimeError(e.to_string()))?;
            let dt = start_time.elapsed().as_secs_f32();
            println!("Step {}/{} done, {:.2}s", timestep_index + 1, args.n_steps, dt);

            // if args.intermediary_images {
            //     save_image(
            //         &vae_model,
            //         &latents,
            //         vae_scale,
            //         bsize,
            //         idx,
            //         // &args.final_image,
            //         args.num_samples,
            //         Some(timestep_index + 1),
            //     ).map_err(|e| LuaError::RuntimeError(e.to_string()));
            // }
        }

        println!(
            "Generating the final image for sample {}/{}.",
            idx + 1,
            args.num_samples
        );
        // let image_bytes = save_image(
        //     &vae_model,
        //     &latents,
        //     vae_scale,
        //     bsize,
        //     idx,
        //     // &args.final_image,
        //     args.num_samples,
        //     None,
        // ).map_err(|e| {
        //     println!("{}", e);
        //     LuaError::RuntimeError(e.to_string())
        // })?;
        let image64 = generate_image(
            &vae_model,
            &latents,
            vae_scale,
            bsize,
            idx,
            args.num_samples,
            None,
        ).map_err(|e| {
            println!("{}", e);
            LuaError::RuntimeError(e.to_string())
        })?;

        images_array.push(image64);
    }

    // let images_table = lua.create_sequence_from(images_array.iter())?;
    let images_table = mlua::Variadic::from_iter(images_array.iter().cloned());
    

    Ok(images_table)
}


// pub fn main(lua: &Lua, table_value: LuaValue) -> LuaResult<()> {
pub fn main(lua: &Lua, table_value: LuaValue) -> LuaResult<mlua::Variadic<String>> {
    let globals = lua.globals();
    let _print: LuaFunction = globals.get("print")?;
    let print = |s: &str| { _print.call::<_, ()>(String::from(s)); };
    print("Starting stable diffusion");
    let args = Args::from_lua(table_value, &lua)?;
    // args.prompt = String::from("A very realistic photo of a rusty robot walking on a sandy beach");

    let images = run(&lua, args)?;
    println!("{:?}", images);
    println!("----------------------------------------");
    println!("----------------------------------------");
    println!("----------------------------------------");
    println!("----------------------------------------");
    println!("----------------------------------------");
    Ok(images)
}
    // format!("data:image/png;base64,{}", base64_string)
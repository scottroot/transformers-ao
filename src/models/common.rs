// #[allow(dead_code)]
// use anyhow::{Result as AnyResult};
// use candle_core::{Tensor, Result as CandleResult, Error as CandleError, DType, Device};
use candle_core::{Tensor, Result as CandleResult};


// pub fn normalize_l2(v: &Tensor) -> AnyResult<Tensor> {
pub fn normalize_l2(v: &Tensor) -> CandleResult<Tensor> {
    Ok(v.broadcast_div(&v.sqr()?.sum_keepdim(1)?.sqrt()?)?)
}

// // pub fn image_preprocess<T: AsRef<std::path::Path>>(path: T) -> anyhow::Result<Tensor> {
// pub fn image_preprocess<T: AsRef<std::path::Path>>(path: T) -> CandleResult<Tensor> {
//     let img = image::io::Reader::open(path)?.decode().map_err(|err| CandleError::wrap(err))?;//?;
//     let (height, width) = (img.height() as usize, img.width() as usize);
//     let height = height - height % 32;
//     let width = width - width % 32;
//     let img = img.resize_to_fill(
//         width as u32,
//         height as u32,
//         image::imageops::FilterType::CatmullRom,
//     );
//     let img = img.to_rgb8();
//     let img = img.into_raw();
//     let img = Tensor::from_vec(img, (height, width, 3), &Device::Cpu).map_err(|err| CandleError::wrap(err))?
//         .permute((2, 0, 1)).map_err(|err| CandleError::wrap(err))?
//         .to_dtype(DType::F32).map_err(|err| CandleError::wrap(err))?
//         .affine(2. / 255., -1.).map_err(|err| CandleError::wrap(err))?
//         .unsqueeze(0).map_err(|err| CandleError::wrap(err))?;
//     Ok(img)
// }

// pub fn load_image<P: AsRef<std::path::Path>>(
//     p: P,
//     resize_longest: Option<usize>,
// ) -> CandleResult<(Tensor, usize, usize)> {
//     let img = image::io::Reader::open(p)?
//         .decode()
//         .map_err(candle_core::Error::wrap)?;
//     let (initial_h, initial_w) = (img.height() as usize, img.width() as usize);
//     let img = match resize_longest {
//         None => img,
//         Some(resize_longest) => {
//             let (height, width) = (img.height(), img.width());
//             let resize_longest = resize_longest as u32;
//             let (height, width) = if height < width {
//                 let h = (resize_longest * height) / width;
//                 (h, resize_longest)
//             } else {
//                 let w = (resize_longest * width) / height;
//                 (resize_longest, w)
//             };
//             img.resize_exact(width, height, image::imageops::FilterType::CatmullRom)
//         }
//     };
//     let (height, width) = (img.height() as usize, img.width() as usize);
//     let img = img.to_rgb8();
//     let data = img.into_raw();
//     let data = Tensor::from_vec(data, (height, width, 3), &Device::Cpu)?.permute((2, 0, 1))?;
//     Ok((data, initial_h, initial_w))
// }
//
// pub fn load_image_and_resize<P: AsRef<std::path::Path>>(
//     p: P,
//     width: usize,
//     height: usize,
// ) -> CandleResult<Tensor> {
//     let img = image::io::Reader::open(p)?
//         .decode()
//         .map_err(candle_core::Error::wrap)?
//         .resize_to_fill(
//             width as u32,
//             height as u32,
//             image::imageops::FilterType::Triangle,
//         );
//     let img = img.to_rgb8();
//     let data = img.into_raw();
//     Tensor::from_vec(data, (width, height, 3), &Device::Cpu)?.permute((2, 0, 1))
// }
//
// /// Saves an image to disk using the image crate, this expects an input with shape
// /// (c, height, width).
// pub fn save_image_file<P: AsRef<std::path::Path>>(img: &Tensor, p: P) -> CandleResult<()> {
//     let p = p.as_ref();
//     let (channel, height, width) = img.dims3().map_err(|err| CandleError::wrap(err))?;
//     if channel != 3 {
//         candle_core::bail!("save_image expects an input of shape (3, height, width)")
//     }
//     let img = img.permute((1, 2, 0))?.flatten_all().map_err(|err| CandleError::wrap(err))?;
//     let pixels = img.to_vec1::<u8>().map_err(|err| CandleError::wrap(err))?;
//     let image: image::ImageBuffer<image::Rgb<u8>, Vec<u8>> =
//         match image::ImageBuffer::from_raw(width as u32, height as u32, pixels) {
//             Some(image) => image,
//             None => candle_core::bail!("error saving image {p:?}"),
//         };
//     image.save(p).map_err(CandleError::wrap)?;
//     Ok(())
// }


//
// pub fn save_image_resize<P: AsRef<std::path::Path>>(
//     img: &Tensor,
//     p: P,
//     h: usize,
//     w: usize,
// ) -> CandleResult<()> {
//     let p = p.as_ref();
//     let (channel, height, width) = img.dims3()?;
//     if channel != 3 {
//         candle_core::bail!("save_image expects an input of shape (3, height, width)")
//     }
//     let img = img.permute((1, 2, 0))?.flatten_all()?;
//     let pixels = img.to_vec1::<u8>()?;
//     let image: image::ImageBuffer<image::Rgb<u8>, Vec<u8>> =
//         match image::ImageBuffer::from_raw(width as u32, height as u32, pixels) {
//             Some(image) => image,
//             None => candle_core::bail!("error saving image {p:?}"),
//         };
//     let image = image::DynamicImage::from(image);
//     let image = image.resize_to_fill(w as u32, h as u32, image::imageops::FilterType::CatmullRom);
//     image.save(p).map_err(candle_core::Error::wrap)?;
//     Ok(())
// }
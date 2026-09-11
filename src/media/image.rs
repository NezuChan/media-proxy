use std::io::Cursor;

use image::imageops::FilterType;
use image::DynamicImage;

use crate::error::AppError;

/// Resizes and crops an image following the original Go service semantics.
///
/// - Equal width/height images are scaled with a nearest-neighbour kernel.
/// - Otherwise the aspect ratio is coerced to a square box and a centre crop
///   is applied (approximating libvips `InterestingCentre`).
/// - The result is always encoded as JPEG with the configured quality.
pub fn process(data: &[u8], width: u32, height: u32, quality: u8) -> Result<Vec<u8>, AppError> {
    let img = image::load_from_memory(data)
        .map_err(|err| AppError::Image(format!("unable to read image: {err}")))?;

    let original_width = img.width();
    let original_height = img.height();

    if original_width == 0 || original_height == 0 {
        return Err(AppError::Image("image has zero dimension".into()));
    }

    let scale_width = f64::from(width) / f64::from(original_width);
    let scale_height = f64::from(height) / f64::from(original_height);
    let scale = scale_width.min(scale_height);

    let processed = if original_width == original_height {
        let new_width = ((f64::from(original_width) * scale).round() as u32).max(1);
        let new_height = ((f64::from(original_height) * scale).round() as u32).max(1);
        img.resize_exact(new_width, new_height, FilterType::Nearest)
    } else {
        let mut crop_width = width;
        let mut crop_height = height;

        if crop_height < crop_width {
            crop_width = crop_height;
        }

        if crop_height > original_height {
            crop_height = original_height;
            crop_width = original_height;
        }

        crop_width = crop_width.min(original_width).max(1);
        crop_height = crop_height.min(original_height).max(1);

        center_crop(&img, crop_width, crop_height)
    };

    encode_jpeg(&processed, quality)
}

fn center_crop(img: &DynamicImage, width: u32, height: u32) -> DynamicImage {
    let x = (img.width().saturating_sub(width)) / 2;
    let y = (img.height().saturating_sub(height)) / 2;
    image::imageops::crop_imm(img, x, y, width, height)
        .to_image()
        .into()
}

fn encode_jpeg(img: &DynamicImage, quality: u8) -> Result<Vec<u8>, AppError> {
    let mut buffer = Cursor::new(Vec::new());
    let mut encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut buffer, quality);
    encoder
        .encode_image(&img.to_rgb8())
        .map_err(|err| AppError::Image(format!("unable to export image: {err}")))?;
    Ok(buffer.into_inner())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_png(width: u32, height: u32) -> Vec<u8> {
        let img = image::RgbImage::from_fn(width, height, |x, y| {
            image::Rgb([(x % 256) as u8, (y % 256) as u8, 128])
        });
        let mut buffer = Cursor::new(Vec::new());
        image::DynamicImage::ImageRgb8(img)
            .write_to(&mut buffer, image::ImageFormat::Png)
            .unwrap();
        buffer.into_inner()
    }

    #[test]
    fn square_image_scales() {
        let png = sample_png(200, 200);
        let out = process(&png, 100, 100, 90).unwrap();
        let decoded = image::load_from_memory(&out).unwrap();
        assert_eq!(decoded.width(), 100);
        assert_eq!(decoded.height(), 100);
    }

    #[test]
    fn non_square_image_crops() {
        let png = sample_png(400, 200);
        let out = process(&png, 100, 100, 90).unwrap();
        let decoded = image::load_from_memory(&out).unwrap();
        assert_eq!(decoded.width(), 100);
        assert_eq!(decoded.height(), 100);
    }

    #[test]
    fn invalid_image_errors() {
        assert!(process(b"not an image", 10, 10, 90).is_err());
    }
}

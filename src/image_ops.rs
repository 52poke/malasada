use bytes::Bytes;

pub fn process_image(data: Bytes, width: Option<u32>) -> Result<(Bytes, String), anyhow::Error> {
    let img = image::load_from_memory(&data)?;

    let mut processed_img = img;

    if let Some(w) = width {
        processed_img = processed_img.resize(w, u32::MAX, image::imageops::FilterType::Lanczos3);
    }

    // Convert DynamicImage to WebP using `webp` crate for lossy compression
    let encoder = webp::Encoder::from_image(&processed_img)
        .map_err(|e| anyhow::anyhow!("WebP encoding failed: {}", e))?;

    // Encode with quality 80 (lossy) to match original sharp defaults
    let memory = encoder.encode(80.0);

    let content_type = "image/webp".to_string();

    // memory is webp::WebPMemory, we can get bytes from it
    // It implements Deref<Target=[u8]>
    Ok((Bytes::from(memory.to_vec()), content_type))
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{DynamicImage, ImageFormat, Rgba, RgbaImage};
    use std::io::Cursor;

    fn png_fixture(width: u32, height: u32) -> Bytes {
        let image = DynamicImage::ImageRgba8(RgbaImage::from_pixel(
            width,
            height,
            Rgba([25, 50, 75, 255]),
        ));
        let mut encoded = Cursor::new(Vec::new());
        image.write_to(&mut encoded, ImageFormat::Png).unwrap();
        Bytes::from(encoded.into_inner())
    }

    #[test]
    fn converts_input_to_webp() {
        let (encoded, content_type) = process_image(png_fixture(4, 2), None).unwrap();

        assert_eq!(content_type, "image/webp");
        assert_eq!(image::guess_format(&encoded).unwrap(), ImageFormat::WebP);
    }

    #[test]
    fn resizes_to_requested_width_preserving_aspect_ratio() {
        let (encoded, _) = process_image(png_fixture(4, 2), Some(2)).unwrap();
        let resized = image::load_from_memory(&encoded).unwrap();

        assert_eq!(resized.width(), 2);
        assert_eq!(resized.height(), 1);
    }

    #[test]
    fn rejects_non_image_input() {
        assert!(process_image(Bytes::from_static(b"not an image"), Some(2)).is_err());
    }
}

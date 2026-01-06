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
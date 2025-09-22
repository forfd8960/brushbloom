pub mod image;

use anyhow::{Result, anyhow};
use photon_rs::{PhotonImage, native::save_image, text::draw_text, transform::resize};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
pub struct ImgMetadata {
    pub fmt: String,
    pub size_in_bytes: u32,
}

#[derive(Serialize)]
pub struct ErrorResponse {
    error: String,
}

#[derive(Serialize)]
struct FileResponse {
    id: String,
    fmt: String,
}

#[derive(Debug, Deserialize)]
pub struct WatermarkRequest {
    text: String,
    position: String,
    font_size: u32,
}

#[derive(Debug, Serialize)]
struct WatermarkResponse {
    new_img_id: String,
}

#[derive(Debug, Deserialize)]
pub struct ResizeImageRequest {
    width: u32,
    height: u32,
    maintain_aspect: bool,
}

#[derive(Debug, Serialize)]
pub struct ResizeImageResponse {
    new_img_id: String,
}

#[derive(Debug, Deserialize)]
pub struct CompressImageRequest {
    quality: u8, // 0-100
}

#[derive(Debug, Serialize)]
pub struct CompressImageResponse {
    new_img_id: String,
}

#[derive(Debug, Deserialize)]
pub struct CorpImageRequest {
    x: u32,
    y: u32,
    width: u32,
    height: u32,
}

#[derive(Debug, Serialize)]
pub struct CorpImageResponse {
    new_img_id: String,
}

// Helper function to add watermark
fn add_watermark_to_image(image: &mut PhotonImage, text: &str, position: &str, font_size: u32) {
    // Determine position coordinates (simplified for example)
    let (x, y) = match position {
        "top-left" => (10, 10),
        "center" => (image.get_width() / 2 - 50, image.get_height() / 2 - 20),
        "bottom-right" => (image.get_width() - 100, image.get_height() - 40),
        _ => (10, 10), // Default to top-left
    };

    // Apply the watermark (photon-rs draw_text applies text with a default font style)
    draw_text(
        image,
        text,
        x as i32,
        y as i32,
        font_size as f32, // photon-rs expects f32 for font size
    );
}

fn resize_image(
    image: &mut PhotonImage,
    width: Option<u32>,
    height: Option<u32>,
    maintain_aspect: bool,
) -> Result<PhotonImage> {
    // Get original dimensions
    let orig_width = image.get_width();
    let orig_height = image.get_height();

    // Determine new dimensions
    let (new_width, new_height) = match (width, height, maintain_aspect) {
        (Some(w), Some(h), false) => (w, h), // Exact dimensions, ignore aspect ratio
        (Some(w), None, _) => {
            // Resize based on width, maintain aspect ratio
            let ratio = w as f32 / orig_width as f32;
            (w, (orig_height as f32 * ratio).round() as u32)
        }
        (None, Some(h), _) => {
            // Resize based on height, maintain aspect ratio
            let ratio = h as f32 / orig_height as f32;
            ((orig_width as f32 * ratio).round() as u32, h)
        }
        (Some(w), Some(h), true) => {
            // Maintain aspect ratio, fit within width and height
            let width_ratio = w as f32 / orig_width as f32;
            let height_ratio = h as f32 / orig_height as f32;
            let ratio = width_ratio.min(height_ratio);
            (
                (orig_width as f32 * ratio).round() as u32,
                (orig_height as f32 * ratio).round() as u32,
            )
        }
        (None, None, _) => {
            return Err(anyhow!("At least one of width or height must be specified"));
        }
    };

    // Resize the image using Lanczos3 filter for high quality
    let resized_image = resize(
        image,
        new_width,
        new_height,
        photon_rs::transform::SamplingFilter::Lanczos3,
    );

    Ok(resized_image)
}

fn save_new_iamge(
    file_path: &str,
    img_meta: &ImgMetadata,
    compressed_image: PhotonImage,
) -> Result<String> {
    let new_image_id = Uuid::new_v4().to_string();
    let output_path = PathBuf::from(format!("{}/{}{}", file_path, new_image_id, img_meta.fmt));

    // Save the modified image
    match save_image(compressed_image, output_path.to_str().unwrap()) {
        Err(e) => return Err(anyhow!("Failed to save image: {}", e)),
        Ok(_) => Ok(new_image_id),
    }
}

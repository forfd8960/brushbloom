pub mod image;

use anyhow::Result;
use photon_rs::{PhotonImage, text::draw_text};
use serde::{Deserialize, Serialize};

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

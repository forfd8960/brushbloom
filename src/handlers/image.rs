use anyhow::{Result, anyhow};
use axum::{
    Json,
    body::Body,
    extract::{Multipart, Path, State},
    http::{HeaderMap, HeaderValue, Response, StatusCode},
    response::IntoResponse,
};
use photon_rs::{PhotonImage, multiple::watermark, native::save_image, open_image};
use std::{fs::File, io::Write, path::PathBuf};
use tracing::{info, warn};
use uuid::Uuid;

use crate::{
    handlers::{
        ErrorResponse, FileResponse, ImgMetadata, WatermarkRequest, WatermarkResponse,
        add_watermark_to_image,
    },
    state::AppState,
};

#[derive(Debug, PartialEq)]
enum ImageFormat {
    Jpeg,
    Png,
    Gif,
    WebP,
    Unknown,
}

impl ImageFormat {
    fn as_str(&self) -> &str {
        match self {
            ImageFormat::Jpeg => ".jpeg",
            ImageFormat::Png => ".png",
            ImageFormat::Gif => ".gif",
            ImageFormat::WebP => ".webp",
            ImageFormat::Unknown => "",
        }
    }
}

fn detect_image_format(content_type: String) -> ImageFormat {
    match content_type.to_lowercase().as_str() {
        "image/jpeg" => ImageFormat::Jpeg,
        "image/png" => ImageFormat::Png,
        "image/gif" => ImageFormat::Gif,
        "image/webp" => ImageFormat::WebP,
        _ => ImageFormat::Unknown,
    }
}

pub async fn upload_image(State(state): State<AppState>, mut mp: Multipart) -> impl IntoResponse {
    let mut file_name = String::new();
    let mut file_data = Vec::new();
    let mut image_type = String::new();

    // Process multipart form data
    while let Some(field) = mp.next_field().await.unwrap_or(None) {
        let field_name = field.name().map(|s| s.to_string());
        info!("field_name: {:?}", field_name);

        match field_name.as_deref() {
            Some("file") => {
                file_name = field
                    .file_name()
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| format!("images-{}", Uuid::new_v4().to_string()));

                image_type = field.content_type().unwrap().to_string();
                info!("uploading file: {}", file_name);

                match field.bytes().await {
                    Ok(data) => file_data = data.to_vec(),
                    Err(_) => {
                        return (
                            StatusCode::BAD_REQUEST,
                            Json(ErrorResponse {
                                error: "Failed to read file data".to_string(),
                            }),
                        )
                            .into_response();
                    }
                }
            }
            _ => {} // Ignore other fields
        }
    }

    info!("file_name: {}", file_name);
    info!("file_data length: {}", file_data.len());

    if file_name.is_empty() || file_data.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "Missing file or filename".to_string(),
            }),
        )
            .into_response();
    }

    write_file(&state, image_type, file_data)
}

fn write_file(state: &AppState, image_type: String, file_data: Vec<u8>) -> Response<Body> {
    let fp = &state.conf.file_path;
    let image_format = detect_image_format(image_type);

    // Generate unique ID and file path
    let file_id = Uuid::new_v4().to_string();
    let file_path = PathBuf::from(format!("{}/{}{}", fp, file_id, image_format.as_str()));

    // Save file to disk
    match File::create(&file_path) {
        Ok(mut file) => {
            info!("writing data to file: {:?}", file_path);

            if let Err(_) = file.write_all(&file_data) {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorResponse {
                        error: "Failed to save file".to_string(),
                    }),
                )
                    .into_response();
            }
        }
        Err(e) => {
            warn!("failed create file: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "Failed to create file".to_string(),
                }),
            )
                .into_response();
        }
    }

    info!("success upload file to: {:?}, {}", file_path, file_id);
    (
        StatusCode::CREATED,
        Json(FileResponse {
            id: file_id,
            fmt: image_format.as_str().to_string(),
        }),
    )
        .into_response()
}

pub async fn get_image(
    headers: HeaderMap,
    State(state): State<AppState>,
    Path(img_id): Path<String>,
) -> impl IntoResponse {
    let file_path = &state.conf.file_path;
    let default_header = &HeaderValue::from_str("application/octet-stream").unwrap();

    let ct = headers
        .get("Content-Type")
        .unwrap_or_else(|| &default_header);

    let ct_value = ct.to_str().unwrap();

    let img_fmts: Vec<&str> = ct_value.split("/").collect();

    info!("get content type: {:?}", img_fmts);

    let img_fmt = detect_image_format(ct_value.to_string());
    if img_fmt == ImageFormat::Unknown {
        return (StatusCode::BAD_REQUEST, format!("unknown image format")).into_response();
    }

    let full_path = format!("{}/{}{}", file_path, img_id, img_fmt.as_str());
    info!("reading: {}", full_path);

    match tokio::fs::read(full_path).await {
        Ok(data) => {
            match Response::builder()
                .header("Content-Type", ct)
                .body(Body::from(data))
            {
                Ok(v) => v,
                Err(e) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Failed to build response: {}", e),
                )
                    .into_response(),
            }
        }
        Err(e) => {
            warn!("failed to read file: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "Failed to read file data".to_string(),
                }),
            )
                .into_response();
        }
    }
}

pub async fn watermark_image(
    State(state): State<AppState>,
    Path(img_id): Path<String>,
    Json(watermk_req): Json<WatermarkRequest>,
) -> impl IntoResponse {
    info!("watermark request: {:?}", watermk_req);

    let file_path = &state.conf.file_path;

    let img_meta_res = get_meta(&state.conf.meta_path, &img_id).await;

    if img_meta_res.is_err() {
        return build_err_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to read file meta".to_string(),
        );
    }

    let img_meta = img_meta_res.unwrap();

    let full_path = format!("{}/{}.{}", file_path, img_id, img_meta.fmt);
    info!("reading: {}", full_path);

    let img_data_res = get_img_data(&full_path).await;
    if img_data_res.is_err() {
        return build_err_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to read image".to_string(),
        );
    }

    let mut photon_img = PhotonImage::new_from_byteslice(img_data_res.unwrap());

    add_watermark_to_image(
        &mut photon_img,
        &watermk_req.text,
        &watermk_req.position,
        watermk_req.font_size,
    );

    // Generate new image ID
    let new_image_id = Uuid::new_v4().to_string();
    let output_path = PathBuf::from(format!("{}/{}.{}", file_path, new_image_id, img_meta.fmt));

    // Save the modified image
    match save_image(photon_img, output_path.to_str().unwrap()) {
        Err(e) => return build_err_response(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
        Ok(_) => {}
    }

    // Return response
    let response = WatermarkResponse {
        new_img_id: new_image_id.clone(),
    };

    (StatusCode::OK, Json(response)).into_response()
}

fn build_err_response(code: StatusCode, msg: String) -> Response<Body> {
    (code, Json(ErrorResponse { error: msg })).into_response()
}

async fn get_meta(meta_path: &str, img_id: &str) -> Result<ImgMetadata> {
    let p = format!("{}/{}", meta_path, img_id);

    match tokio::fs::read(p).await {
        Ok(data) => serde_json::from_slice(&data).map_err(|e| anyhow!("{}", e)),
        Err(e) => Err(anyhow!("{}", e)),
    }
}

async fn get_img_data(img_path: &str) -> Result<Vec<u8>> {
    match tokio::fs::read(img_path).await {
        Ok(data) => Ok(data),
        Err(e) => Err(anyhow!("{}", e)),
    }
}

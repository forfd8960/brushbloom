use std::{fs::File, io::Write, path::PathBuf};

use crate::{
    handlers::{ErrorResponse, FileResponse},
    state::AppState,
};

use axum::{
    Json,
    extract::{Multipart, State},
    http::StatusCode,
    response::IntoResponse,
};
use uuid::Uuid;

pub async fn upload_image(State(state): State<AppState>, mut mp: Multipart) -> impl IntoResponse {
    let mut filename = String::new();
    let mut file_data = Vec::new();

    // Process multipart form data
    while let Some(field) = mp.next_field().await.unwrap_or(None) {
        let field_name = field.name().unwrap_or("").to_string();

        match field_name.as_str() {
            "file" => {
                if let Some(file_filename) = field.file_name() {
                    filename = sanitize_filename(file_filename);
                }

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

    if filename.is_empty() || file_data.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "Missing file or filename".to_string(),
            }),
        )
            .into_response();
    }

    let fp = &state.conf.file_path;

    // Generate unique ID and file path
    let file_id = Uuid::new_v4().to_string();
    let file_path = PathBuf::from(format!("{}/{}", fp, file_id));

    // Save file to disk
    match File::create(&file_path) {
        Ok(mut file) => {
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
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "Failed to create file".to_string(),
                }),
            )
                .into_response();
        }
    }

    (StatusCode::CREATED, Json(FileResponse { id: file_id })).into_response()
}

fn sanitize_filename(filename: &str) -> String {
    filename
        .chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            _ => c,
        })
        .collect()
}

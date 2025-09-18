use serde::Serialize;

pub mod upload;

#[derive(Serialize)]
pub struct ErrorResponse {
    error: String,
}

#[derive(Serialize)]
struct FileResponse {
    id: String,
}

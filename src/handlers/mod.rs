use serde::Serialize;

pub mod image;

#[derive(Serialize)]
pub struct ErrorResponse {
    error: String,
}

#[derive(Serialize)]
struct FileResponse {
    id: String,
    fmt: String,
}

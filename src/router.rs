use std::{fs::File, io::Write, path::PathBuf};

use anyhow::Result;
use axum::{
    Json, Router,
    extract::{Multipart, State},
    http::StatusCode,
    response::IntoResponse,
    routing::post,
};

use crate::{handlers::upload::upload_image, state::AppState};

pub fn routers(app_state: AppState) -> Result<Router> {
    let router = Router::new()
        .route("/upload", post(upload_image))
        .with_state(app_state);

    Ok(router)
}

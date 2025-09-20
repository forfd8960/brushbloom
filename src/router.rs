use anyhow::Result;
use axum::{
    Router,
    routing::{get, post},
};

use crate::{
    handlers::image::{get_image, upload_image, watermark_image},
    state::AppState,
};

pub fn routers(app_state: AppState) -> Result<Router> {
    let router = Router::new()
        .route("/api/images/upload", post(upload_image))
        .route("/api/images/{img_id}", get(get_image))
        .route("/api/images/{img_id}/watermark", post(watermark_image))
        .with_state(app_state);

    Ok(router)
}

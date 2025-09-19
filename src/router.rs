use anyhow::Result;
use axum::{
    Router,
    routing::{get, post},
};

use crate::{handlers::upload::upload_image, state::AppState};

pub fn routers(app_state: AppState) -> Result<Router> {
    let router = Router::new()
        .route("/api/images/upload", post(upload_image))
        .with_state(app_state);

    Ok(router)
}

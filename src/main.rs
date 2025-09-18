use anyhow::Result;
use brushbloom::{
    router,
    state::{AppConfig, AppState},
};

use tokio::net::TcpListener;
use tracing::{info, level_filters::LevelFilter};
use tracing_subscriber::{Layer as _, fmt::Layer, layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<()> {
    let layer = Layer::new().with_filter(LevelFilter::INFO);
    tracing_subscriber::registry().with(layer).init();

    let app_conf = AppConfig::new("config.toml")?;
    info!("app_conf: {:?}", app_conf);

    let app_state = AppState::new(app_conf);
    info!("app_state: {:?}", app_state);

    let app = router::routers(app_state)?;
    let listener = TcpListener::bind("0.0.0.0:8080").await?;

    axum::serve(listener, app).await?;
    Ok(())
}

use anyhow::Result;
use brushbloom::state::{AppConfig, AppState};

use tracing::{info, level_filters::LevelFilter};
use tracing_subscriber::{Layer as _, fmt::Layer, layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<()> {
    let layer = Layer::new().with_filter(LevelFilter::INFO);
    tracing_subscriber::registry().with(layer).init();

    let app_conf = AppConfig::new("config.toml")?;
    info!("app_conf: {:?}", app_conf);

    let app_state = AppState::new(app_conf);

    info!("app state: {:?}", app_state);
    Ok(())
}

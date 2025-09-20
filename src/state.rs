use anyhow::{Result, anyhow};
use bytes::BytesMut;
use serde::Deserialize;
use std::{fs::File, io::Read, ops::Deref, sync::Arc};

#[derive(Debug, Clone)]
pub struct AppState {
    pub inner: Arc<AppStateInner>,
}

#[derive(Debug, Clone)]
pub struct AppStateInner {
    pub conf: AppConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    pub max_file_size: u64,
    pub file_path: String,
    pub meta_path: String,
}

impl AppConfig {
    pub fn new(path: &str) -> Result<Self> {
        let mut file = File::open(path)?;
        let mut buf = BytesMut::with_capacity(4096).to_vec();
        let _ = file.read_to_end(&mut buf)?;

        match toml::from_slice(&buf) {
            Ok(v) => Ok(v),
            Err(e) => return Err(anyhow!("{}", e)),
        }
    }
}

impl AppState {
    pub fn new(config: AppConfig) -> Self {
        Self {
            inner: Arc::new(AppStateInner { conf: config }),
        }
    }
}

impl Deref for AppState {
    type Target = AppStateInner;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

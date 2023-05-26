use std::{
    fs, io,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};
use tendermint_rpc::WebSocketClientUrl;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Config {
    pub chains: Chains,
    pub database: Database,
    pub metrics: Metrics,
}

impl Config {
    pub fn load<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let content = fs::read_to_string(path)?;
        let config =
            toml::from_str(&content).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        Ok(config)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Chains {
    pub endpoints: Vec<Endpoint>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Endpoint {
    pub name: String,
    pub url: WebSocketClientUrl,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Database {
    pub path: PathBuf,
}

#[derive(Copy, Clone, Debug, Deserialize, Serialize)]
pub struct Metrics {
    pub enabled: bool,
    pub port: u16,
}

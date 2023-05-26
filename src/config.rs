use std::{
    fs, io,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};
use tendermint::chain;
use tendermint_rpc::{client::CompatMode as CometVersion, WebSocketClientUrl};

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
    pub name: chain::Id,
    pub url: WebSocketClientUrl,

    #[serde(
        default = "crate::config::default::comet_version",
        with = "crate::config::comet_version"
    )]
    pub comet_version: CometVersion,
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

mod default {
    use super::*;

    pub fn comet_version() -> CometVersion {
        CometVersion::V0_34
    }
}

mod comet_version {
    use super::*;
    use serde::{Deserialize, Serializer};

    pub fn serialize<S>(version: &CometVersion, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let version = match version {
            CometVersion::V0_37 => "0.37",
            CometVersion::V0_34 => "0.34",
        };

        serializer.serialize_str(version)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<CometVersion, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let version = String::deserialize(deserializer)?;

        match version.as_str() {
            "0.37" => Ok(CometVersion::V0_37),
            "0.34" => Ok(CometVersion::V0_34),
            _ => Err(serde::de::Error::custom(format!(
                "invalid CometBFT version: {}, available: 0.34, 0.37",
                version
            ))),
        }
    }
}
pub mod collect;
pub mod db;
pub mod metrics;
pub mod msg;

use std::path::PathBuf;

use clap::Parser;

use metrics::Metrics;
use tendermint_rpc::WebSocketClientUrl;
use tracing::error;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

/// Collect and analyze txs containing IBC messages, export the collected metrics for Prometheus
#[derive(clap::Parser)]
struct App {
    /// Tendermint WebSocket URL
    #[clap(long = "ws", default_value = "wss://rpc.osmosis.zone/websocket")]
    ws_url: WebSocketClientUrl,

    /// Path to the SQLite database file, will be created if not existing
    #[clap(long = "db", default_value = "osmosis.db")]
    db_path: PathBuf,

    /// Port on which to serve the Prometheus metrics, at `http://0.0.0.0:PORT/metrics`.
    /// If not set, then the metrics won't be served
    #[clap(long = "metrics", value_name = "PORT")]
    metrics: Option<u16>,
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    let app = App::parse();

    setup_tracing();

    let (metrics, registry) = Metrics::new();

    if let Some(port) = app.metrics {
        tokio::spawn(metrics::run(port, registry));
    }

    if let Err(e) = collect::run(app.ws_url, app.db_path, metrics).await {
        error!("{e}");
    }

    Ok(())
}

fn setup_tracing() {
    use tracing_subscriber::prelude::*;
    use tracing_subscriber::{filter::EnvFilter, fmt};

    let fmt_layer = fmt::layer().with_target(false);

    let filter_layer = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new("collect_packets=info"))
        .unwrap();

    tracing_subscriber::registry()
        .with(filter_layer)
        .with(fmt_layer)
        .init();
}

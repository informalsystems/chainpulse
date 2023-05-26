use std::net::SocketAddr;

use axum::{extract::State, routing::get, Router, Server};
use prometheus::{
    register_int_gauge_vec_with_registry, Encoder, IntGaugeVec, Registry, TextEncoder,
};
use tendermint::chain;
use tracing::info;

type GaugeVec = IntGaugeVec;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

#[derive(Clone)]
pub struct Metrics {
    /// Counts the number of IBC packets that are effected
    /// Labels: ['chain_id', 'src_channel', 'src_port', 'dst_channel', 'dst_port', 'signer', 'memo']
    ibc_effected_packets: GaugeVec,
    /// Counts the number of IBC packets that are not effected
    /// Labels: ['chain_id', 'src_channel', 'src_port', 'dst_channel', 'dst_port', 'signer', 'memo']
    ibc_uneffected_packets: GaugeVec,
    /// Counts the number of times a signer gets frontrun by the same original signer
    /// Labels: ['chain_id', 'src_channel', 'src_port', 'dst_channel', 'dst_port', 'signer', 'frontrunned_by', 'memo', 'effected_memo']
    ibc_frontrun_counter: GaugeVec,
}

impl Metrics {
    pub fn new() -> (Self, Registry) {
        let registry = Registry::new();

        let ibc_effected_packets = register_int_gauge_vec_with_registry!(
            "ibc_effected_packets",
            "Counts the number of IBC packets that are effected",
            &[
                "chain_id",
                "src_channel",
                "src_port",
                "dst_channel",
                "dst_port",
                "signer",
                "memo",
            ],
            registry
        )
        .unwrap();

        let ibc_uneffected_packets = register_int_gauge_vec_with_registry!(
            "ibc_uneffected_packets",
            "Counts the number of IBC packets that are not effected",
            &[
                "chain_id",
                "src_channel",
                "src_port",
                "dst_channel",
                "dst_port",
                "signer",
                "memo"
            ],
            registry
        )
        .unwrap();

        let ibc_frontrun_counter = register_int_gauge_vec_with_registry!(
            "ibc_frontrun_counter",
            "Counts the number of times a signer gets frontrun by the same original signer",
            &[
                "chain_id",
                "src_channel",
                "src_port",
                "dst_channel",
                "dst_port",
                "signer",
                "frontrunned_by",
                "memo",
                "effected_memo"
            ],
            registry
        )
        .unwrap();

        (
            Self {
                ibc_effected_packets,
                ibc_uneffected_packets,
                ibc_frontrun_counter,
            },
            registry,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn ibc_effected_packets(
        &self,
        chain_id: &chain::Id,
        src_channel: &str,
        src_port: &str,
        dst_channel: &str,
        dst_port: &str,
        signer: &str,
        memo: &str,
    ) {
        self.ibc_effected_packets
            .with_label_values(&[
                chain_id.as_ref(),
                src_channel,
                src_port,
                dst_channel,
                dst_port,
                signer,
                memo,
            ])
            .inc();
    }

    #[allow(clippy::too_many_arguments)]
    pub fn ibc_uneffected_packets(
        &self,
        chain_id: &chain::Id,
        src_channel: &str,
        src_port: &str,
        dst_channel: &str,
        dst_port: &str,
        signer: &str,
        memo: &str,
    ) {
        self.ibc_uneffected_packets
            .with_label_values(&[
                chain_id.as_ref(),
                src_channel,
                src_port,
                dst_channel,
                dst_port,
                signer,
                memo,
            ])
            .inc();
    }

    #[allow(clippy::too_many_arguments)]
    pub fn ibc_frontrun_counter(
        &self,
        chain_id: &chain::Id,
        src_channel: &str,
        src_port: &str,
        dst_channel: &str,
        dst_port: &str,
        signer: &str,
        frontrunned_by: &str,
        memo: &str,
        effected_memo: &str,
    ) {
        self.ibc_frontrun_counter
            .with_label_values(&[
                chain_id.as_ref(),
                src_channel,
                src_port,
                dst_channel,
                dst_port,
                signer,
                frontrunned_by,
                memo,
                effected_memo,
            ])
            .inc();
    }
}

pub async fn run(port: u16, registry: Registry) -> Result<()> {
    let app = Router::new()
        .route("/metrics", get(get_metrics))
        .with_state(registry);

    let server =
        Server::bind(&SocketAddr::from(([0, 0, 0, 0], port))).serve(app.into_make_service());

    info!("Metrics server listening at http://localhost:{port}/metrics");
    server.await?;

    Ok(())
}

pub async fn get_metrics(registry: State<Registry>) -> String {
    let mut buffer = vec![];
    let encoder = TextEncoder::new();

    let metric_families = registry.gather();
    encoder.encode(&metric_families, &mut buffer).unwrap();

    String::from_utf8(buffer).unwrap()
}

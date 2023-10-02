use std::net::SocketAddr;

use axum::{extract::State, routing::get, Router, Server};
use prometheus::{
    register_int_counter_vec_with_registry, register_int_gauge_vec_with_registry, Encoder,
    IntCounterVec, IntGaugeVec, Registry, TextEncoder,
};
use tendermint::chain;
use tracing::info;

type GaugeVec = IntGaugeVec;
type CounterVec = IntCounterVec;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

#[derive(Clone)]
pub struct Metrics {
    /// The number of IBC packets that are effected
    /// Labels: ['chain_id', 'src_channel', 'src_port', 'dst_channel', 'dst_port', 'signer', 'memo']
    ibc_effected_packets: CounterVec,

    /// The number of IBC packets that are not effected
    /// Labels: ['chain_id', 'src_channel', 'src_port', 'dst_channel', 'dst_port', 'signer', 'memo']
    ibc_uneffected_packets: CounterVec,

    /// The number of times a signer gets frontrun by the original signer
    /// Labels: ['chain_id', 'src_channel', 'src_port', 'dst_channel', 'dst_port', 'signer', 'frontrunned_by', 'memo', 'effected_memo']
    ibc_frontrun_counter: CounterVec,

    /// The number of stuck packets on an IBC channel
    /// Labels: ['src_chain', 'dst_chain', 'src_channel']
    ibc_stuck_packets: GaugeVec,

    /// The number of chains being monitored
    chainpulse_chains: GaugeVec,

    /// The number of txs processed
    /// Labels: ['chain_id']
    chainpulse_txs: CounterVec,

    /// The number of packets processed
    /// Labels: ['chain_id']
    chainpulse_packets: CounterVec,

    /// The number of times we had to reconnect to the WebSocket
    /// Labels: ['chain_id']
    chainpulse_reconnects: CounterVec,

    /// The number of times the WebSocket connection timed out
    /// Labels: 'chain_id']
    chainpulse_timeouts: CounterVec,

    /// The number of times we encountered an error
    /// Labels: ['chain_id']
    chainpulse_errors: CounterVec,
}

impl Metrics {
    pub fn new() -> (Self, Registry) {
        let registry = Registry::new();

        let ibc_effected_packets = register_int_counter_vec_with_registry!(
            "ibc_effected_packets",
            "The number of IBC packets that have been relayed and were effected",
            &[
                "chain_id",
                "src_channel",
                "src_port",
                "dst_channel",
                "dst_port",
                "signer",
                "memo",
            ],
            registry,
        )
        .unwrap();

        let ibc_uneffected_packets = register_int_counter_vec_with_registry!(
            "ibc_uneffected_packets",
            "The number of IBC packets that were relayed but not effected",
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

        let ibc_frontrun_counter = register_int_counter_vec_with_registry!(
            "ibc_frontrun_counter",
            "The number of times a signer gets frontrun by the original signer",
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

        let ibc_stuck_packets = register_int_gauge_vec_with_registry!(
            "ibc_stuck_packets",
            "The number of packets stuck on an IBC channel",
            &["src_chain", "dst_chain", "src_channel"],
            registry
        )
        .unwrap();

        let chainpulse_chains = register_int_gauge_vec_with_registry!(
            "chainpulse_chains",
            "The number of chains being monitored",
            &[],
            registry
        )
        .unwrap();

        let chainpulse_txs = register_int_counter_vec_with_registry!(
            "chainpulse_txs",
            "The number of txs processed",
            &["chain_id"],
            registry
        )
        .unwrap();

        let chainpulse_packets = register_int_counter_vec_with_registry!(
            "chainpulse_packets",
            "The number of packets processed",
            &["chain_id"],
            registry
        )
        .unwrap();

        let chainpulse_reconnects = register_int_counter_vec_with_registry!(
            "chainpulse_reconnects",
            "The number of times we had to reconnect to the WebSocket",
            &["chain_id"],
            registry
        )
        .unwrap();

        let chainpulse_timeouts = register_int_counter_vec_with_registry!(
            "chainpulse_timeouts",
            "The number of times the WebSocket connection timed out",
            &["chain_id"],
            registry
        )
        .unwrap();

        let chainpulse_errors = register_int_counter_vec_with_registry!(
            "chainpulse_errors",
            "The number of times an error was encountered",
            &["chain_id"],
            registry
        )
        .unwrap();

        (
            Self {
                ibc_effected_packets,
                ibc_uneffected_packets,
                ibc_frontrun_counter,
                ibc_stuck_packets,
                chainpulse_chains,
                chainpulse_txs,
                chainpulse_packets,
                chainpulse_reconnects,
                chainpulse_timeouts,
                chainpulse_errors,
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

    pub fn ibc_stuck_packets(
        &self,
        src_chain: &str,
        dst_chain: &str,
        src_channel: &str,
        value: i64,
    ) {
        self.ibc_stuck_packets
            .with_label_values(&[src_chain, dst_chain, src_channel])
            .set(value);
    }

    pub fn chainpulse_chains(&self) {
        self.chainpulse_chains.with_label_values(&[]).inc();
    }

    pub fn chainpulse_txs(&self, chain_id: &chain::Id) {
        self.chainpulse_txs
            .with_label_values(&[chain_id.as_ref()])
            .inc();
    }

    pub fn chainpulse_packets(&self, chain_id: &chain::Id) {
        self.chainpulse_packets
            .with_label_values(&[chain_id.as_ref()])
            .inc();
    }

    pub fn chainpulse_reconnects(&self, chain_id: &chain::Id) {
        self.chainpulse_reconnects
            .with_label_values(&[chain_id.as_ref()])
            .inc();
    }

    pub fn chainpulse_timeouts(&self, chain_id: &chain::Id) {
        self.chainpulse_timeouts
            .with_label_values(&[chain_id.as_ref()])
            .inc();
    }

    pub fn chainpulse_errors(&self, chain_id: &chain::Id) {
        self.chainpulse_errors
            .with_label_values(&[chain_id.as_ref()])
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

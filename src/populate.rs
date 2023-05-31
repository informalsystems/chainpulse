use std::{collections::HashSet, time::Instant};

use futures::StreamExt;
use sqlx::SqlitePool;
use tendermint::chain;
use tracing::{error_span, info};

use crate::{
    db::{PacketRow, TxRow},
    metrics::Metrics,
};

pub async fn run(chain: &chain::Id, pool: &SqlitePool, metrics: &Metrics) -> crate::Result<()> {
    let _span = error_span!("populate", %chain).entered();

    info!("Populating metrics...");

    let start = Instant::now();

    let mut packets =
            sqlx::query_as::<_, PacketRow>(
            "SELECT packets.* FROM packets LEFT JOIN txs ON packets.tx_id = txs.id WHERE txs.chain = ? ORDER BY id")
                .bind(chain.as_str())
                .fetch(pool);

    let mut ids = HashSet::new();

    while let Some(Ok(packet)) = packets.next().await {
        metrics.chainpulse_packets(chain);

        let tx = sqlx::query_as::<_, TxRow>("SELECT * FROM txs WHERE id = ? LIMIT 1")
            .bind(packet.tx_id)
            .fetch_one(pool)
            .await?;

        if !ids.contains(&tx.id) {
            metrics.chainpulse_txs(chain);
            ids.insert(tx.id);
        }

        if packet.effected {
            metrics.ibc_effected_packets(
                chain,
                &packet.src_channel,
                &packet.src_port,
                &packet.dst_channel,
                &packet.dst_port,
                &packet.signer,
                &tx.memo,
            );
        } else {
            let effected_tx = sqlx::query_as::<_, TxRow>("SELECT * FROM txs WHERE id = ? LIMIT 1")
                .bind(packet.effected_tx)
                .fetch_one(pool)
                .await?;

            metrics.ibc_uneffected_packets(
                chain,
                &packet.src_channel,
                &packet.src_port,
                &packet.dst_channel,
                &packet.dst_port,
                &packet.signer,
                &tx.memo,
            );

            metrics.ibc_frontrun_counter(
                chain,
                &packet.src_channel,
                &packet.src_port,
                &packet.dst_channel,
                &packet.dst_port,
                &packet.signer,
                &packet.effected_signer.unwrap_or_default(),
                &tx.memo,
                &effected_tx.memo,
            )
        }
    }

    let elapsed = start.elapsed();
    info!("Populated metrics in {elapsed:?}");

    Ok(())
}

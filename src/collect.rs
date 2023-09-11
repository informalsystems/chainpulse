use std::time::Duration;

use futures::StreamExt;
use ibc_proto::cosmos::tx::v1beta1::Tx;
use prost::Message;
use sqlx::SqlitePool;
use tendermint::{
    block::Height,
    chain::{self, Id as ChainId},
    crypto::Sha256,
};
use tendermint_rpc::{
    client::CompatMode,
    event::{Event, EventData},
    Client, SubscriptionClient, WebSocketClient, WebSocketClientUrl,
};
use tokio::time;
use tracing::{error, info, warn, Instrument};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

type Pool = SqlitePool;

use crate::{
    db::{PacketRow, TxRow},
    metrics::Metrics,
    msg::Msg,
};

const NEWBLOCK_TIMEOUT: Duration = Duration::from_secs(60);
const DISCONNECT_AFTER_BLOCKS: usize = 100;

#[derive(Copy, Clone, Debug, thiserror::Error)]
pub enum Outcome {
    #[error("Timeout after {0:?} waiting for a NewBlock event")]
    Timeout(Duration),

    #[error("Disconnecting after {0} blocks")]
    BlockElapsed(usize),
}

pub async fn run(
    chain_id: chain::Id,
    compat_mode: CompatMode,
    ws_url: WebSocketClientUrl,
    db: Pool,
    metrics: Metrics,
) -> Result<()> {
    loop {
        let task = collect(&chain_id, compat_mode, &ws_url, &db, &metrics);

        match task.await {
            Ok(outcome) => warn!("{outcome}"),
            Err(e) => {
                metrics.chainpulse_errors(&chain_id);

                error!("{e}")
            }
        }

        metrics.chainpulse_reconnects(&chain_id);

        info!("Reconnecting in 5 seconds...");
        time::sleep(Duration::from_secs(5)).await;
    }
}

async fn collect(
    chain_id: &chain::Id,
    compat_mode: CompatMode,
    ws_url: &WebSocketClientUrl,
    db: &Pool,
    metrics: &Metrics,
) -> Result<Outcome> {
    info!("Connecting to {ws_url}...");
    let (client, driver) = WebSocketClient::builder(ws_url.clone())
        .compat_mode(compat_mode)
        .build()
        .await?;

    tokio::spawn(driver.run());

    info!("Subscribing to NewBlock events...");
    let mut subscription = client.subscribe(queries::new_block()).await?;

    info!("Waiting for new blocks...");

    let mut count: usize = 0;

    loop {
        let next_event = time::timeout(NEWBLOCK_TIMEOUT, subscription.next()).await;
        let next_event = match next_event {
            Ok(next_event) => next_event,
            Err(_) => {
                metrics.chainpulse_timeouts(chain_id);
                return Ok(Outcome::Timeout(NEWBLOCK_TIMEOUT));
            }
        };

        count += 1;

        let Some(Ok(event)) = next_event else {
            continue;
        };

        let (chain_id, client, pool, metrics) = (
            chain_id.clone(),
            client.clone(),
            db.clone(),
            metrics.clone(),
        );

        tokio::spawn(
            async move {
                if let Err(e) = on_new_block(client, pool, event, &metrics).await {
                    metrics.chainpulse_errors(&chain_id);

                    error!("{e}");
                }
            }
            .in_current_span(),
        );

        if count >= DISCONNECT_AFTER_BLOCKS {
            return Ok(Outcome::BlockElapsed(count));
        }
    }
}

async fn on_new_block(
    client: WebSocketClient,
    db: Pool,
    event: Event,
    metrics: &Metrics,
) -> Result<()> {
    let EventData::NewBlock {
        block: Some(block), ..
    } = event.data
    else {
        return Ok(());
    };

    let height = block.header.height;
    let chain_id = block.header.chain_id;

    info!("New block at height {}", block.header.height);

    let block = client.block(height).await?;

    for tx in &block.block.data {
        metrics.chainpulse_txs(&chain_id);

        let tx = Tx::decode(tx.as_slice())?;
        let tx_row = insert_tx(&db, &chain_id, height, &tx).await?;

        let msgs = tx.body.ok_or("missing tx body")?.messages;

        for msg in msgs {
            let type_url = msg.type_url.clone();

            if let Ok(msg) = Msg::decode(msg) {
                if msg.is_ibc() {
                    info!("    {msg}");

                    if msg.is_relevant() {
                        process_msg(&db, &chain_id, &tx_row, &type_url, msg, metrics).await?;
                    }
                }
            }
        }
    }

    Ok(())
}

async fn process_msg(
    pool: &Pool,
    chain_id: &ChainId,
    tx_row: &TxRow,
    type_url: &str,
    msg: Msg,
    metrics: &Metrics,
) -> Result<()> {
    let Some(packet) = msg.packet() else {
        return Ok(());
    };

    metrics.chainpulse_packets(chain_id);

    tracing::debug!(
        "    Packet #{} in tx {} ({}) - {}",
        packet.sequence,
        tx_row.id,
        tx_row.hash,
        tx_row.memo
    );

    let query = r#"
        SELECT * FROM packets
        WHERE   src_channel = ? 
            AND src_port = ? 
            AND dst_channel = ? 
            AND dst_port = ? 
            AND sequence = ?
            AND msg_type_url = ?
            LIMIT 1
    "#;

    let existing: Option<PacketRow> = sqlx::query_as(query)
        .bind(&packet.source_channel)
        .bind(&packet.source_port)
        .bind(&packet.destination_channel)
        .bind(&packet.destination_port)
        .bind(packet.sequence as i64)
        .bind(type_url)
        .fetch_optional(pool)
        .await?;

    if let Some(existing) = &existing {
        let effected_tx: TxRow = sqlx::query_as("SELECT * FROM txs WHERE id = ? LIMIT 1")
            .bind(existing.tx_id)
            .fetch_one(pool)
            .await?;

        tracing::debug!(
            "        Frontrun by tx {} ({}) - {}",
            existing.tx_id,
            effected_tx.hash,
            effected_tx.memo
        );

        metrics.ibc_uneffected_packets(
            chain_id,
            &packet.source_channel,
            &packet.source_port,
            &packet.destination_channel,
            &packet.destination_port,
            msg.signer().unwrap_or(""),
            &tx_row.memo,
        );

        metrics.ibc_frontrun_counter(
            chain_id,
            &packet.source_channel,
            &packet.source_port,
            &packet.destination_channel,
            &packet.destination_port,
            msg.signer().unwrap_or(""),
            &existing.signer,
            &tx_row.memo,
            &effected_tx.memo,
        );
    } else {
        metrics.ibc_effected_packets(
            chain_id,
            &packet.source_channel,
            &packet.source_port,
            &packet.destination_channel,
            &packet.destination_port,
            msg.signer().unwrap_or(""),
            &tx_row.memo,
        );
    }

    let query = r#"
        INSERT OR IGNORE INTO packets
            (tx_id, sequence, src_channel, src_port, dst_channel, dst_port,
            msg_type_url, signer, effected, effected_signer, effected_tx, created_at)
        VALUES
            (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, datetime('now'))
    "#;

    sqlx::query(query)
        .bind(tx_row.id)
        .bind(packet.sequence as i64)
        .bind(&packet.source_channel)
        .bind(&packet.source_port)
        .bind(&packet.destination_channel)
        .bind(&packet.destination_port)
        .bind(type_url)
        .bind(msg.signer())
        .bind(existing.is_none())
        .bind(existing.as_ref().map(|row| &row.signer))
        .bind(existing.as_ref().map(|row| row.tx_id))
        .execute(pool)
        .await?;

    Ok(())
}

async fn insert_tx(db: &Pool, chain_id: &ChainId, height: Height, tx: &Tx) -> Result<TxRow> {
    let query = r#"
        INSERT OR IGNORE INTO txs (chain, height, hash, memo, created_at)
        VALUES (?, ?, ?, ?, datetime('now'))
    "#;

    let bytes = tx.encode_to_vec();
    let hash = tendermint::crypto::default::Sha256::digest(&bytes);
    let hash = subtle_encoding::hex::encode_upper(hash);
    let hash = String::from_utf8_lossy(&hash);

    let height = height.value() as i64;

    let memo = tx
        .body
        .as_ref()
        .map(|body| body.memo.to_string())
        .unwrap_or_default();

    sqlx::query(query)
        .bind(chain_id.as_str())
        .bind(height)
        .bind(&hash)
        .bind(memo)
        .execute(db)
        .await?;

    let tx: TxRow =
        sqlx::query_as("SELECT * FROM txs WHERE chain = ? AND height = ? AND hash = ? LIMIT 1")
            .bind(chain_id.as_str())
            .bind(height)
            .bind(hash)
            .fetch_one(db)
            .await?;

    Ok(tx)
}

mod queries {
    use tendermint_rpc::query::{EventType, Query};

    pub fn new_block() -> Query {
        Query::from(EventType::NewBlock)
    }
}

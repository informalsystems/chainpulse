use core::fmt;
use std::{
    path::{Path, PathBuf},
    time::Duration,
};

use futures::StreamExt;
use ibc_proto::cosmos::tx::v1beta1::Tx;
use prost::Message;
use sqlx::SqlitePool;
use tendermint::{block::Height, chain::Id as ChainId, crypto::Sha256};
use tendermint_rpc::{
    client::CompatMode,
    event::{Event, EventData},
    Client, SubscriptionClient, WebSocketClient, WebSocketClientUrl,
};
use tokio::time;
use tracing::{error, info};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

type Pool = SqlitePool;

use crate::{
    db::{self, PacketRow, TxRow},
    metrics::Metrics,
    msg::{decode_msg, print_msg, Msg},
};

const NEWBLOCK_TIMEOUT: Duration = Duration::from_secs(60);
const MAX_BLOCKS: usize = 100;

#[derive(Copy, Clone, Debug, thiserror::Error)]
struct TimeoutError(Duration);

impl fmt::Display for TimeoutError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Timeout after {:?} waiting for a NewBlock event", self.0)
    }
}

#[derive(Copy, Clone, Debug, thiserror::Error)]
struct BlockElapsed(usize);

impl fmt::Display for BlockElapsed {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Disconnecting after {} blocks", self.0)
    }
}

pub async fn run(ws_url: WebSocketClientUrl, db_path: PathBuf, metrics: Metrics) -> Result<()> {
    loop {
        let task = collect(ws_url.clone(), &db_path, &metrics);

        if let Err(e) = task.await {
            error!("{}", e);
        }

        info!("Reconnecting in 5 seconds...");
        time::sleep(Duration::from_secs(5)).await;
    }
}

async fn collect(ws_url: WebSocketClientUrl, db_path: &Path, metrics: &Metrics) -> Result<()> {
    let options = sqlx::sqlite::SqliteConnectOptions::new()
        .filename(db_path)
        .create_if_missing(true)
        .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal);

    let pool = SqlitePool::connect_with(options).await?;
    db::setup(&pool).await;

    info!("Connecting...");
    let (client, driver) = WebSocketClient::builder(ws_url)
        .compat_mode(CompatMode::V0_34)
        .build()
        .await?;

    tokio::spawn(driver.run());

    info!("Subscribing to NewBlock events...");
    let mut subscription = client.subscribe(queries::new_block()).await?;

    info!("Waiting for new blocks...");

    let mut count: usize = 0;

    loop {
        let next_event = time::timeout(NEWBLOCK_TIMEOUT, subscription.next())
            .await
            .map_err(|_| TimeoutError(NEWBLOCK_TIMEOUT))?;

        count += 1;

        let Some(Ok(event)) = next_event else { continue; };

        let (client, pool, metrics) = (client.clone(), pool.clone(), metrics.clone());

        tokio::spawn(async {
            if let Err(e) = on_new_block(client, pool, event, metrics).await {
                error!("{e}");
            }
        });

        if count >= MAX_BLOCKS {
            return Err(BlockElapsed(count).into());
        }
    }
}

async fn on_new_block(
    client: WebSocketClient,
    pool: Pool,
    event: Event,
    metrics: Metrics,
) -> Result<()> {
    let EventData::NewBlock { block: Some(block), .. } = event.data else { return Ok(()) };

    let height = block.header.height;
    let chain_id = block.header.chain_id;

    info!("New block at height {}", block.header.height);

    let block = client.block(height).await?;

    for tx in &block.block.data {
        let tx = Tx::decode(tx.as_slice())?;
        let tx_row = insert_tx(&pool, &chain_id, height, &tx).await?;

        let msgs = tx.body.ok_or("missing tx body")?.messages;

        for msg in msgs {
            let type_url = msg.type_url.clone();

            if let Ok(msg) = decode_msg(msg) {
                if msg.is_ibc() {
                    print_msg(&msg);

                    if msg.is_relevant() {
                        process_msg(&pool, &chain_id, &tx_row, &type_url, msg, &metrics).await?;
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
    let Some(packet) = msg.packet() else { return Ok(()) };

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

async fn insert_tx(pool: &Pool, chain_id: &ChainId, height: Height, tx: &Tx) -> Result<TxRow> {
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
        .execute(pool)
        .await?;

    let tx: TxRow =
        sqlx::query_as("SELECT * FROM txs WHERE chain = ? AND height = ? AND hash = ? LIMIT 1")
            .bind(chain_id.as_str())
            .bind(height)
            .bind(hash)
            .fetch_one(pool)
            .await?;

    Ok(tx)
}

mod queries {
    use tendermint_rpc::query::{EventType, Query};

    pub fn new_block() -> Query {
        Query::from(EventType::NewBlock)
    }
}

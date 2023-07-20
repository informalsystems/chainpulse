use std::path::Path;

use sqlx::{sqlite::SqliteConnectOptions, SqlitePool};
use time::PrimitiveDateTime;

use crate::Result;

#[derive(Clone, Debug, sqlx::FromRow)]
pub struct TxRow {
    pub id: i64,
    pub chain: String,
    pub height: i64,
    pub hash: String,
    pub memo: String,
    pub created_at: PrimitiveDateTime,
}

#[derive(Clone, Debug, sqlx::FromRow)]
pub struct PacketRow {
    pub id: i64,
    pub tx_id: i64,
    pub sequence: i64,
    pub src_channel: String,
    pub src_port: String,
    pub dst_channel: String,
    pub dst_port: String,
    pub msg_type_url: String,
    pub signer: String,
    pub effected: bool,
    pub effected_signer: Option<String>,
    pub effected_tx: Option<i64>,
    pub created_at: PrimitiveDateTime,
}

pub async fn connect(path: &Path) -> Result<SqlitePool> {
    let options = SqliteConnectOptions::new()
        .filename(path)
        .create_if_missing(true)
        .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal);

    let pool = SqlitePool::connect_with(options).await?;

    Ok(pool)
}

pub async fn setup(pool: &SqlitePool) {
    create_tables(pool).await;
    create_indexes(pool).await;
}

pub async fn create_tables(pool: &SqlitePool) {
    const TABLES: &[&str] = &[
        r#"
        CREATE TABLE IF NOT EXISTS txs (
            id           INTEGER PRIMARY KEY AUTOINCREMENT,
            chain        TEXT    NOT NULL,
            height       INTEGER NOT NULL,
            hash         TEXT    NOT NULL,
            memo         TEXT    NOT NULL,
            created_at   TEXT    NOT NULL
        );
        "#,
        r#"
        CREATE TABLE IF NOT EXISTS packets (
            id                  INTEGER PRIMARY KEY AUTOINCREMENT,
            tx_id               INTEGER NOT NULL REFERENCES txs (id),
            sequence            INTEGER NOT NULL,
            src_channel         TEXT    NOT NULL,
            src_port            TEXT    NOT NULL,
            dst_channel         TEXT    NOT NULL,
            dst_port            TEXT    NOT NULL,
            msg_type_url        TEXT    NOT NULL,
            signer              TEXT,
            effected            BOOL    NOT NULL,
            effected_signer     TEXT,
            created_at          TEXT    NOT NULL
        );
        "#,
    ];

    for table in TABLES {
        sqlx::query(table).execute(pool).await.unwrap();
    }

    const MIGRATIONS: &[&str] =
        &["ALTER TABLE packets ADD COLUMN effected_tx INTEGER REFERENCES txs (id);"];

    for migration in MIGRATIONS {
        run_migration(pool, migration).await;
    }

    create_indexes(pool).await;
}

async fn create_indexes(pool: &SqlitePool) {
    const INDEXES: &[&str] = &[
        "CREATE UNIQUE INDEX IF NOT EXISTS txs_unique          ON txs (chain, hash);",
        "CREATE        INDEX IF NOT EXISTS txs_chain           ON txs (chain);",
        "CREATE        INDEX IF NOT EXISTS txs_hash            ON txs (hash);",
        "CREATE        INDEX IF NOT EXISTS txs_memo            ON txs (memo);",
        "CREATE        INDEX IF NOT EXISTS txs_height          ON txs (height);",
        "CREATE        INDEX IF NOT EXISTS txs_created_at      ON txs (created_at);",
        "CREATE        INDEX IF NOT EXISTS packets_tx_id       ON packets(tx_id);",
        "CREATE        INDEX IF NOT EXISTS packets_signer      ON packets (signer);",
        "CREATE        INDEX IF NOT EXISTS packets_src_channel ON packets (src_channel);",
        "CREATE        INDEX IF NOT EXISTS packets_dst_channel ON packets (dst_channel);",
        "CREATE        INDEX IF NOT EXISTS packets_effected    ON packets (effected);",
        "CREATE        INDEX IF NOT EXISTS packets_effected_tx ON packets (effected_tx);",
    ];

    for index in INDEXES {
        sqlx::query(index).execute(pool).await.unwrap();
    }
}

async fn run_migration(pool: &SqlitePool, migration: &str) {
    if (sqlx::query(migration).execute(pool).await).is_err() {
        tracing::debug!("Migration fail to apply, perhaps it was not needed: {migration}");
    }
}

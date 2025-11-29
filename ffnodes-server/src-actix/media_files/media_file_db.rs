use crate::media_files::MediaFile;
use anyhow::Result;
use log::LevelFilter;
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode};
use sqlx::{ConnectOptions, Executor, SqlitePool, Transaction};
use std::path::Path;
use std::time::UNIX_EPOCH;

pub async fn initialize() -> Result<()> {
    let pool = open_pool().await?;

    // Media files table
    pool.execute(
        r#"CREATE TABLE IF NOT EXISTS `media_files`
(
    path                 TEXT    PRIMARY KEY NOT NULL UNIQUE,
    scanned_size         INTEGER NOT NULL,
    size                 INTEGER          DEFAULT NULL,
    scanned_bit_rate     INTEGER NOT NULL,
    bit_rate             INTEGER          DEFAULT NULL,
    duration             REAL    NOT NULL,
    width                INTEGER NOT NULL,
    height               INTEGER NOT NULL,
    frames               INTEGER NOT NULL,
    last_modified        INTEGER NOT NULL,
    encoding_complexity  INTEGER NOT NULL DEFAULT 0,
    retry_count          INTEGER NOT NULL DEFAULT 0,
    processed            INTEGER NOT NULL DEFAULT 0
)"#,
    )
    .await?;

    // Encoding jobs table
    pool.execute(
        r#"CREATE TABLE IF NOT EXISTS `encoding_jobs`
(
    id              TEXT    PRIMARY KEY NOT NULL,
    media_file_path TEXT    NOT NULL,
    status          TEXT    NOT NULL,
    priority        INTEGER NOT NULL,
    assigned_client TEXT             DEFAULT NULL,
    assigned_at     INTEGER          DEFAULT NULL,
    started_at      INTEGER          DEFAULT NULL,
    completed_at    INTEGER          DEFAULT NULL,
    error_message   TEXT             DEFAULT NULL,
    output_path     TEXT             DEFAULT NULL,
    output_size     INTEGER          DEFAULT NULL,
    output_bitrate  INTEGER          DEFAULT NULL,
    created_at      INTEGER NOT NULL,
    FOREIGN KEY (media_file_path) REFERENCES media_files(path) ON DELETE CASCADE,
    FOREIGN KEY (assigned_client) REFERENCES clients(id) ON DELETE SET NULL
)"#,
    )
    .await?;

    // Clients table
    pool.execute(
        r#"CREATE TABLE IF NOT EXISTS `clients`
(
    id             TEXT    PRIMARY KEY NOT NULL,
    display_name   TEXT    NOT NULL,
    computer_name  TEXT    NOT NULL,
    machine_id     TEXT             DEFAULT NULL,
    connected_at   INTEGER NOT NULL,
    last_heartbeat INTEGER NOT NULL,
    disconnected_at INTEGER         DEFAULT NULL
)"#,
    )
    .await?;

    // Migration: Add machine_id column if it doesn't exist (for existing databases)
    let _ = pool
        .execute("ALTER TABLE clients ADD COLUMN machine_id TEXT DEFAULT NULL")
        .await;
    // Ignore error if column already exists

    // Migration: Add average_speed column if it doesn't exist (for existing databases)
    let _ = pool
        .execute("ALTER TABLE encoding_jobs ADD COLUMN average_speed REAL DEFAULT NULL")
        .await;
    // Ignore error if column already exists

    // Migration: Add current_phase column if it doesn't exist (for existing databases)
    let _ = pool
        .execute("ALTER TABLE encoding_jobs ADD COLUMN current_phase TEXT DEFAULT NULL")
        .await;
    // Ignore error if column already exists

    // Create indexes for client lookups
    pool.execute("CREATE INDEX IF NOT EXISTS idx_clients_computer_name ON clients(computer_name)")
        .await?;

    // Unique index on machine_id (when not NULL)
    pool.execute(
        "CREATE UNIQUE INDEX IF NOT EXISTS idx_clients_machine_id ON clients(machine_id) WHERE machine_id IS NOT NULL"
    )
    .await?;

    // Encoding progress table
    pool.execute(
        r#"CREATE TABLE IF NOT EXISTS `encoding_progress`
(
    job_id         TEXT    NOT NULL,
    frame          INTEGER NOT NULL,
    fps            REAL    NOT NULL,
    bitrate        TEXT    NOT NULL,
    speed          TEXT    NOT NULL,
    updated_at     INTEGER NOT NULL,
    PRIMARY KEY (job_id),
    FOREIGN KEY (job_id) REFERENCES encoding_jobs(id) ON DELETE CASCADE
)"#,
    )
    .await?;

    Ok(())
}

impl MediaFile {
    #[allow(dead_code)]
    pub async fn insert(
        &self,
        transaction: &mut Transaction<'_, sqlx::sqlite::Sqlite>,
    ) -> Result<()> {
        sqlx::query(
            r#"INSERT INTO media_files
			(path, scanned_size, scanned_bit_rate, duration, width, height, frames, last_modified, encoding_complexity, retry_count, processed)
			VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"#,
        )
        .bind(self.path.to_string_lossy().to_string())
        .bind(self.scanned_size as i64)
        .bind(self.scanned_bit_rate as i64)
        .bind(self.duration as f64)
        .bind(self.width as i64)
        .bind(self.height as i64)
        .bind(self.frames as i64)
        .bind(self.last_modified as i64)
        .bind(self.encoding_complexity as i64)
        .bind(self.retry_count as i64)
        .bind(self.processed)
        .execute(&mut **transaction)
        .await?;

        Ok(())
    }

    pub async fn insert_direct(&self, pool: &SqlitePool) -> Result<()> {
        sqlx::query(
            r#"INSERT INTO media_files
			(path, scanned_size, scanned_bit_rate, duration, width, height, frames, last_modified, encoding_complexity, retry_count, processed)
			VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"#,
        )
        .bind(self.path.to_string_lossy().to_string())
        .bind(self.scanned_size as i64)
        .bind(self.scanned_bit_rate as i64)
        .bind(self.duration as f64)
        .bind(self.width as i64)
        .bind(self.height as i64)
        .bind(self.frames as i64)
        .bind(self.last_modified as i64)
        .bind(self.encoding_complexity as i64)
        .bind(self.retry_count as i64)
        .bind(self.processed)
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn does_path_need_probing(path: impl AsRef<Path>, pool: &SqlitePool) -> Result<bool> {
        let path = path.as_ref();
        let last_modified = path.metadata()?.modified()?;
        let db_last_modified =
            sqlx::query_scalar::<_, i64>(r#"SELECT last_modified FROM media_files WHERE path = ?"#)
                .bind(path.to_string_lossy().to_string().clone())
                .fetch_optional(pool)
                .await?
                .unwrap_or(0);
        Ok(last_modified.duration_since(UNIX_EPOCH)?.as_secs() > (db_last_modified as u64))
    }
}

pub async fn open_pool() -> Result<SqlitePool> {
    let options = SqliteConnectOptions::new()
        .journal_mode(SqliteJournalMode::Wal)
        .foreign_keys(true)
        .filename("app.db")
        .log_statements(LevelFilter::Trace)
        .create_if_missing(true);
    Ok(SqlitePool::connect_with(options).await?)
}

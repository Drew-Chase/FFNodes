use sqlx::{ConnectOptions, Executor, SqlitePool, Transaction};
use anyhow::Result;
use log::LevelFilter;
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode};
use crate::media_files::MediaFile;

pub async fn initialize() -> Result<()> {
	let pool = open_pool().await?;
	pool.execute(
		r#"CREATE TABLE IF NOT EXISTS `media_files`
(
    id               INTEGER PRIMARY KEY,
    path             TEXT    NOT NULL,
    scanned_size     INTEGER NOT NULL,
    size             INTEGER          DEFAULT NULL,
    scanned_bit_rate INTEGER NOT NULL,
    bit_rate         INTEGER          DEFAULT NULL,
    duration         INTEGER NOT NULL,
    width            INTEGER NOT NULL,
    height           INTEGER NOT NULL,
    frames           INTEGER NOT NULL,
    processed        INTEGER NOT NULL DEFAULT 0
)"#,
	)
	    .await?;

	Ok(())
}


impl MediaFile{
	pub async fn insert(&self, transaction: &mut Transaction<'_,sqlx::sqlite::Sqlite>) -> Result<()> {
		sqlx::query(
			r#"INSERT INTO media_files
			(path, scanned_size, scanned_bit_rate, duration, width, height, frames, processed)
			VALUES (?, ?, ?, ?, ?, ?, ?, ?)"#
		)
			.bind(self.path.to_string_lossy().to_string())
			.bind(self.scanned_size as i64)
			.bind(self.scanned_bit_rate as i64)
			.bind(self.duration as i64)
			.bind(self.width as i64)
			.bind(self.height as i64)
			.bind(self.frames as i64)
			.bind(self.processed)
			.execute(&mut **transaction)
			.await?;

		Ok(())
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
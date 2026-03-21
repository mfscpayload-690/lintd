use crate::pmal::{PackageSource, RemovalRecord};
use chrono::Utc;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePool, SqlitePoolOptions};
use std::str::FromStr;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DbError {
    #[error("Database error: {0}")]
    SqlxError(#[from] sqlx::Error),

    #[error("Migration error: {0}")]
    MigrationError(String),
}

pub struct Database {
    pool: SqlitePool,
}

impl Database {
    pub async fn new() -> Result<Self, DbError> {
        let data_dir = dirs::data_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("."))
            .join("lintd");

        std::fs::create_dir_all(&data_dir).ok();
        let db_path = data_dir.join("lintd.db");
        let db_url = format!("sqlite:{}?mode=rwc", db_path.to_string_lossy());

        let options = SqliteConnectOptions::from_str(&db_url)
            .map_err(|e| DbError::MigrationError(e.to_string()))?
            .create_if_missing(true);

        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect_with(options)
            .await?;

        let db = Self { pool };
        db.run_migrations().await?;
        Ok(db)
    }

    async fn run_migrations(&self) -> Result<(), DbError> {
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS removal_history (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                package_name TEXT NOT NULL,
                source TEXT NOT NULL,
                removed_at TEXT NOT NULL,
                space_recovered_bytes INTEGER NOT NULL DEFAULT 0,
                command_executed TEXT NOT NULL DEFAULT ''
            )",
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn record_removal(
        &self,
        package_name: &str,
        source: &PackageSource,
        space_recovered: u64,
        command: &str,
    ) -> Result<(), DbError> {
        let now = Utc::now().to_rfc3339();
        let source_str = source.to_string();

        sqlx::query(
            "INSERT INTO removal_history (package_name, source, removed_at, space_recovered_bytes, command_executed)
             VALUES (?, ?, ?, ?, ?)",
        )
        .bind(package_name)
        .bind(&source_str)
        .bind(&now)
        .bind(space_recovered as i64)
        .bind(command)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn get_removal_history(&self) -> Result<Vec<RemovalRecord>, DbError> {
        let rows = sqlx::query_as::<_, RemovalRow>(
            "SELECT id, package_name, source, removed_at, space_recovered_bytes, command_executed
             FROM removal_history ORDER BY removed_at DESC",
        )
        .fetch_all(&self.pool)
        .await?;

        let records = rows
            .into_iter()
            .map(|row| {
                let source = match row.source.as_str() {
                    "pacman" => PackageSource::Pacman,
                    "aur" => PackageSource::Aur,
                    "apt" => PackageSource::Apt,
                    "dnf" => PackageSource::Dnf,
                    "flatpak" => PackageSource::Flatpak,
                    "snap" => PackageSource::Snap,
                    "appimage" => PackageSource::AppImage,
                    "apk" => PackageSource::Apk,
                    "nix" => PackageSource::Nix,
                    _ => PackageSource::Manual,
                };
                let removed_at = chrono::DateTime::parse_from_rfc3339(&row.removed_at)
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now());

                RemovalRecord {
                    id: row.id,
                    package_name: row.package_name,
                    source,
                    removed_at,
                    space_recovered_bytes: row.space_recovered_bytes as u64,
                    command_executed: row.command_executed,
                }
            })
            .collect();

        Ok(records)
    }

    pub async fn get_flatpak_zero_space_history(&self) -> Result<Vec<(i64, String)>, DbError> {
        let rows = sqlx::query_as::<_, FlatpakBackfillRow>(
            "SELECT id, command_executed
             FROM removal_history
             WHERE source = 'flatpak' AND space_recovered_bytes = 0",
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|row| (row.id, row.command_executed))
            .collect())
    }

    pub async fn update_removal_space_recovered(
        &self,
        id: i64,
        space_recovered: u64,
    ) -> Result<(), DbError> {
        sqlx::query(
            "UPDATE removal_history
             SET space_recovered_bytes = ?
             WHERE id = ?",
        )
        .bind(space_recovered as i64)
        .bind(id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}

#[derive(sqlx::FromRow)]
struct RemovalRow {
    id: i64,
    package_name: String,
    source: String,
    removed_at: String,
    space_recovered_bytes: i64,
    command_executed: String,
}

#[derive(sqlx::FromRow)]
struct FlatpakBackfillRow {
    id: i64,
    command_executed: String,
}

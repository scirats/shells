mod codec;
mod schema;

use async_trait::async_trait;
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePool, SqlitePoolOptions};
use std::path::PathBuf;

use kernel::{Profile, ProfileStore, ShellName, StoreError};

use codec::{decode, encode};
use schema::SHELLS_TABLE;

pub struct SqliteProfileStore {
    pool: SqlitePool,
}

impl SqliteProfileStore {
    pub async fn open(database: PathBuf) -> Result<Self, StoreError> {
        if let Some(parent) = database.parent() {
            std::fs::create_dir_all(parent).map_err(internal)?;
        }
        let options = SqliteConnectOptions::new()
            .filename(&database)
            .create_if_missing(true)
            .journal_mode(SqliteJournalMode::Wal)
            .foreign_keys(true);
        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect_with(options)
            .await
            .map_err(internal)?;

        sqlx::query(SHELLS_TABLE)
            .execute(&pool)
            .await
            .map_err(internal)?;

        Ok(Self { pool })
    }
}

#[async_trait]
impl ProfileStore for SqliteProfileStore {
    async fn get(&self, name: &ShellName) -> Result<Option<Profile>, StoreError> {
        let row: Option<(String,)> = sqlx::query_as("SELECT profile FROM shells WHERE name = ?1")
            .bind(name.as_str())
            .fetch_optional(&self.pool)
            .await
            .map_err(internal)?;
        row.map(|(json,)| decode(&json)).transpose()
    }

    async fn put(&self, name: &ShellName, profile: &Profile) -> Result<(), StoreError> {
        let json = encode(profile)?;
        sqlx::query(
            "INSERT INTO shells(name, profile) VALUES(?1, ?2)
             ON CONFLICT(name) DO UPDATE SET profile = excluded.profile, updated_at = datetime('now')",
        )
        .bind(name.as_str())
        .bind(json)
        .execute(&self.pool)
        .await
        .map_err(internal)?;
        Ok(())
    }

    async fn delete(&self, name: &ShellName) -> Result<(), StoreError> {
        sqlx::query("DELETE FROM shells WHERE name = ?1")
            .bind(name.as_str())
            .execute(&self.pool)
            .await
            .map_err(internal)?;
        Ok(())
    }

    async fn list(&self) -> Result<Vec<(ShellName, Profile)>, StoreError> {
        let rows: Vec<(String, String)> =
            sqlx::query_as("SELECT name, profile FROM shells ORDER BY name")
                .fetch_all(&self.pool)
                .await
                .map_err(internal)?;

        let mut entries = Vec::with_capacity(rows.len());
        for (name, json) in rows {
            let name = ShellName::parse(&name).map_err(|_| {
                StoreError::Internal(format!("stored shell name '{name}' is invalid"))
            })?;
            entries.push((name, decode(&json)?));
        }
        Ok(entries)
    }
}

pub(crate) fn internal(error: impl std::fmt::Display) -> StoreError {
    StoreError::Internal(error.to_string())
}

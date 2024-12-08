// Path: src/migrations.rs

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use std::sync::Arc;
use surrealdb::engine::remote::ws::Client;
use surrealdb::Surreal;
use tracing::{info, error, instrument};
use crate::telemetry::TelemetryManager;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum MigrationError {
    #[error("Database error: {0}")]
    DatabaseError(#[from] surrealdb::Error),

    #[error("Migration failed: {0}")]
    MigrationFailed(String),
}

pub type MigrationResult<T> = std::result::Result<T, MigrationError>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Migration {
    pub version: i32,
    pub name: String,
    pub description: String,
    pub up: String,
    pub down: String,
    pub applied_at: Option<DateTime<Utc>>,
}

pub struct MigrationManager {
    db: Arc<Surreal<Client>>,
    telemetry: Arc<TelemetryManager>,
    migrations: Vec<Migration>,
}

impl MigrationManager {
    pub async fn new(db: Arc<Surreal<Client>>, telemetry: Arc<TelemetryManager>) -> MigrationResult<Self> {
        Ok(Self {
            db,
            telemetry,
            migrations: Vec::new(),
        })
    }

    pub fn add_migration(&mut self, migration: Migration) {
        self.migrations.push(migration);
    }

    #[instrument(name = "run_pending_migrations", skip(self))]
    pub async fn run_pending_migrations(&self) -> MigrationResult<()> {
        let current_version = self.get_current_version().await?;
        
        for migration in self.migrations.iter().filter(|m| m.version > current_version) {
            self.apply_migration(migration).await?;
        }

        Ok(())
    }

    #[instrument(name = "apply_migration", skip(self), fields(version = %migration.version, name = %migration.name))]
    async fn apply_migration(&self, migration: &Migration) -> MigrationResult<()> {
        info!("Applying migration {} - {}", migration.version, migration.name);
        
        // Clone the necessary data upfront
        let version = migration.version;
        let name = migration.name.clone();
        let description = migration.description.clone();
        let up_query = migration.up.clone();

        self.telemetry.record_metric(
            "migration_apply_details".to_string(),
            1.0,
            vec![
                ("version".to_string(), version.to_string()),
                ("name".to_string(), name.clone()),
            ],
        );

        // Execute migration using cloned query
        self.db.query(&up_query)
            .await
            .map_err(|e| MigrationError::MigrationFailed(format!("Failed to apply migration: {}", e)))?;

        // Record migration using already cloned data
        self.db.query("CREATE migration SET version = $version, name = $name, description = $description, applied_at = time::now()")
            .bind(("version", version))
            .bind(("name", name))
            .bind(("description", description))
            .await
            .map_err(MigrationError::DatabaseError)?;

        info!("Migration {} applied successfully", migration.version);
        Ok(())
    }

    #[instrument(name = "rollback", skip(self), fields(target_version = %target_version))]
    pub async fn rollback(&self, target_version: i32) -> MigrationResult<()> {
        let current_version = self.get_current_version().await?;
        
        self.telemetry.record_metric(
            "migration_rollback".to_string(),
            1.0,
            vec![
                ("from_version".to_string(), current_version.to_string()),
                ("to_version".to_string(), target_version.to_string()),
            ],
        );

        for migration in self.migrations.iter().filter(|m| m.version > target_version).rev() {
            self.rollback_migration(migration).await?;
        }

        Ok(())
    }

    #[instrument(name = "rollback_migration", skip(self), fields(version = %migration.version, name = %migration.name))]
    async fn rollback_migration(&self, migration: &Migration) -> MigrationResult<()> {
        info!("Rolling back migration {} - {}", migration.version, migration.name);
        
        self.telemetry.record_metric(
            "migration_rollback_single".to_string(),
            1.0,
            vec![
                ("version".to_string(), migration.version.to_string()),
                ("name".to_string(), migration.name.clone()),
            ],
        );

        // Execute rollback
        self.db.query(&migration.down)
            .await
            .map_err(|e| MigrationError::MigrationFailed(format!("Failed to rollback migration: {}", e)))?;

        // Remove migration record
        let version = migration.version;
        self.db.query("DELETE FROM migration WHERE version = $version")
            .bind(("version", version))
            .await
            .map_err(MigrationError::DatabaseError)?;

        info!("Migration {} rolled back successfully", migration.version);
        Ok(())
    }

    pub async fn get_current_version(&self) -> MigrationResult<i32> {
        let mut response = self.db.query("SELECT version FROM migration ORDER BY version DESC LIMIT 1").await
            .map_err(MigrationError::DatabaseError)?;
        
        let version = response.take::<Option<i32>>(0)
            .map_err(MigrationError::DatabaseError)?
            .unwrap_or(0);
        
        Ok(version)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use surrealdb::engine::remote::ws::Ws;

    #[derive(Debug)]
    struct TestConfig {
        url: String,
        namespace: String,
        database: String,
        username: String,
        password: String,
    }

    async fn setup_test_migration() -> MigrationResult<MigrationManager> {
        let config = TestConfig {
            url: "ws://localhost:8000".to_string(),
            namespace: "test".to_string(),
            database: "test".to_string(),
            username: "root".to_string(),
            password: "root".to_string(),
        };

        let client = Surreal::new::<Ws>(&config.url).await.unwrap();
        client.signin(surrealdb::opt::auth::Root {
            username: &config.username,
            password: &config.password,
        }).await.unwrap();
        client.use_ns(&config.namespace).use_db(&config.database).await.unwrap();

        let telemetry_manager = TelemetryManager::init()
            .await
            .map_err(|e| MigrationError::MigrationFailed(e.to_string()))?;
        let manager = MigrationManager::new(
            Arc::new(client), 
            Arc::new(telemetry_manager)
        ).await?;
        
        Ok(manager)
    }
}

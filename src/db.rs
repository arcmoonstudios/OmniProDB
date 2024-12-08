// Path: src/db.rs

use std::sync::Arc;
use surrealdb::engine::remote::ws::{Client, Ws};
use surrealdb::Surreal;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DatabaseError {
    #[error("Database error: {0}")]
    DatabaseError(#[from] surrealdb::Error),
    #[error("Invalid input: {0}")]
    InvalidInput(String),
}

pub type DatabaseResult<T> = Result<T, DatabaseError>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub namespace: String,
    pub database: String,
    pub username: String,
    pub password: String,
}

pub struct DatabaseManager {
    client: Arc<Surreal<Client>>,
}

impl DatabaseManager {
    pub async fn new(config: DatabaseConfig) -> DatabaseResult<Self> {
        let client = Surreal::new::<Ws>(&config.url).await?;
        
        client
            .signin(surrealdb::opt::auth::Root {
                username: &config.username,
                password: &config.password,
            })
            .await?;
        
        client.use_ns(&config.namespace).use_db(&config.database).await?;
        
        Ok(Self { 
            client: Arc::new(client)
        })
    }

    pub async fn get_connection(&self) -> DatabaseResult<Arc<Surreal<Client>>> {
        Ok(self.client.clone())
    }

    pub async fn health_check(&self) -> DatabaseResult<()> {
        self.client
            .health()
            .await
            .map_err(|e| DatabaseError::DatabaseError(e))?;
        Ok(())
    }
}

use surrealdb::Surreal;
use std::sync::Arc;
use serde::{Serialize, Deserialize};
use thiserror::Error;
use surrealdb::engine::remote::ws::Client;

#[derive(Debug, Serialize, Deserialize)]
pub struct Dataset {
    pub id: String,
    pub name: String,
    pub description: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Model {
    pub id: String,
    pub name: String,
    pub description: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Error)]
pub enum SurrealMLError {
    #[error("Database error: {0}")]
    DatabaseError(#[from] surrealdb::Error),
    #[error("IO error: {0}")]
    IOError(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, SurrealMLError>;

pub struct SurrealMLStorage {
    client: Arc<Surreal<Client>>,
}

impl SurrealMLStorage {
    pub fn new(client: Arc<Surreal<Client>>) -> Self {
        Self { client }
    }

    pub async fn store_dataset(&self, id: String, dataset: Dataset, data: Vec<u8>) -> Result<()> {
        // Store dataset data
        self.client
            .query("CREATE type::thing('dataset_data', $id) SET data = $data")
            .bind(("id", id.clone()))
            .bind(("data", data))
            .await?;

        // Store dataset metadata
        self.client
            .query("CREATE type::thing('dataset', $id) SET name = $name, description = $description, created_at = $created_at, data_pointer = $data_pointer")
            .bind(("id", id.clone()))
            .bind(("name", dataset.name))
            .bind(("description", dataset.description))
            .bind(("created_at", dataset.created_at))
            .bind(("data_pointer", format!("dataset_data:{}", id)))
            .await?;

        Ok(())
    }

    pub async fn get_dataset(&self, id: String) -> Result<Option<(Dataset, Vec<u8>)>> {
        let dataset = self.client
            .query("SELECT * FROM type::thing('dataset', $id)")
            .bind(("id", id.clone()))
            .await?
            .take(0)?;

        if let Some(dataset) = dataset {
            // Get dataset data
            let data = self.client
                .query("SELECT data FROM type::thing('dataset_data', $id)")
                .bind(("id", id))
                .await?
                .take(0)?;

            Ok(Some((dataset, data)))
        } else {
            Ok(None)
        }
    }

    pub async fn store_model(&self, id: String, model: Model, weights: Vec<u8>) -> Result<()> {
        // Store model weights
        self.client
            .query("CREATE type::thing('model_data', $id) SET weights = $weights")
            .bind(("id", id.clone()))
            .bind(("weights", weights))
            .await?;

        // Store model metadata
        self.client
            .query("CREATE type::thing('model', $id) SET name = $name, description = $description, created_at = $created_at, model_pointer = $model_pointer")
            .bind(("id", id.clone()))
            .bind(("name", model.name))
            .bind(("description", model.description))
            .bind(("created_at", model.created_at))
            .bind(("model_pointer", format!("model_data:{}", id)))
            .await?;

        Ok(())
    }

    pub async fn get_model(&self, id: String) -> Result<Option<(Model, Vec<u8>)>> {
        let model = self.client
            .query("SELECT * FROM type::thing('model', $id)")
            .bind(("id", id.clone()))
            .await?
            .take(0)?;

        if let Some(model) = model {
            // Get model weights
            let weights = self.client
                .query("SELECT weights FROM type::thing('model_data', $id)")
                .bind(("id", id))
                .await?
                .take(0)?;

            Ok(Some((model, weights)))
        } else {
            Ok(None)
        }
    }

    pub async fn list_datasets(&self, limit: i64, offset: i64) -> Result<Vec<Dataset>> {
        let datasets = self.client
            .query("SELECT * FROM dataset ORDER BY created_at DESC LIMIT $limit OFFSET $offset")
            .bind(("limit", limit))
            .bind(("offset", offset))
            .await?
            .take(0)?;

        Ok(datasets)
    }

    pub async fn list_models(&self, limit: i64, offset: i64) -> Result<Vec<Model>> {
        let models = self.client
            .query("SELECT * FROM model ORDER BY created_at DESC LIMIT $limit OFFSET $offset")
            .bind(("limit", limit))
            .bind(("offset", offset))
            .await?
            .take(0)?;

        Ok(models)
    }
}
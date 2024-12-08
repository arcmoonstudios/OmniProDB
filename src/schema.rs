use crate::db::DatabaseManager;
use std::error::Error;
use surrealdb::sql::Thing;
use tracing::{info, warn};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldDefinition {
    pub name: String,
    pub field_type: String,
    pub required: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexDefinition {
    pub name: String,
    pub fields: Vec<String>,
    pub unique: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableDefinition {
    pub name: String,
    pub fields: Vec<FieldDefinition>,
    pub indexes: Vec<IndexDefinition>,
}

pub async fn init_schema(db: &DatabaseManager) -> Result<(), Box<dyn Error>> {
    info!("Initializing database schema...");

    // Create user table
    create_user_table(db).await?;

    info!("Schema initialization completed successfully");
    Ok(())
}

async fn create_user_table(db: &DatabaseManager) -> Result<(), Box<dyn Error>> {
    info!("Creating user table...");

    // Define table
    let table_def = r#"
    DEFINE TABLE user SCHEMAFULL;
    
    DEFINE FIELD id ON user TYPE string;
    DEFINE FIELD email ON user TYPE string;
    DEFINE FIELD name ON user TYPE string;
    DEFINE FIELD password_hash ON user TYPE string;
    DEFINE FIELD role ON user TYPE string;
    DEFINE FIELD created_at ON user TYPE datetime;
    DEFINE FIELD updated_at ON user TYPE datetime;
    
    DEFINE INDEX user_email ON user FIELDS email UNIQUE;
    "#;

    // Execute table definition
    let conn = db.get_connection().await?;
    match conn.query(table_def).await {
        Ok(_) => {
            info!("User table created successfully");
            Ok(())
        }
        Err(e) => {
            warn!("Error creating user table: {}", e);
            // If table already exists, we can ignore the error
            if e.to_string().contains("already exists") {
                info!("User table already exists, skipping creation");
                Ok(())
            } else {
                Err(Box::new(e))
            }
        }
    }
}

// Helper function to check if a table exists
pub async fn table_exists(db: &DatabaseManager, table_name: String) -> Result<bool, Box<dyn std::error::Error>> {
    let conn = db.get_connection().await?;
    let mut response = conn
        .query("INFO FOR TABLE $table")
        .bind(("table", table_name))
        .await?;
    
    let result: Vec<Thing> = response.take(0)?;
    Ok(!result.is_empty())
}
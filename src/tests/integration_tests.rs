#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::DatabaseManager;
    use crate::schema::TableDefinition;
    use surrealdb::engine::remote::ws::Client;
    use surrealdb::Surreal;

    async fn setup_test_db() -> DatabaseManager {
        let config = DatabaseConfig {
            url: "ws://localhost:8000".to_string(),
            namespace: "test".to_string(),
            database: "test".to_string(),
        };
        DatabaseManager::new(config).await.unwrap()
    }

    #[tokio::test]
    async fn test_database_connection() {
        let db = setup_test_db().await;
        assert!(db.connect().await.is_ok());
    }

    #[tokio::test]
    async fn test_create_table() {
        let db = setup_test_db().await;
        db.connect().await.unwrap();

        let table = TableDefinition {
            name: "test_table".to_string(),
            fields: vec![],
            indexes: vec![],
        };

        assert!(db.create_table(&table).await.is_ok());
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::DatabaseManager;
    use std::time::Instant;
    use tokio::time::Duration;

    const CONCURRENT_CONNECTIONS: usize = 100;
    const OPERATIONS_PER_CONNECTION: usize = 1000;

    async fn setup_performance_test() -> DatabaseManager {
        let config = DatabaseConfig {
            url: "ws://localhost:8000".to_string(),
            namespace: "perf_test".to_string(),
            database: "perf_test".to_string(),
        };
        DatabaseManager::new(config).await.unwrap()
    }

    #[tokio::test]
    async fn test_concurrent_connections() {
        let db = setup_performance_test().await;
        let start = Instant::now();
        
        let handles: Vec<_> = (0..CONCURRENT_CONNECTIONS)
            .map(|_| {
                let db_clone = db.clone();
                tokio::spawn(async move {
                    for _ in 0..OPERATIONS_PER_CONNECTION {
                        db_clone.health_check().await.unwrap();
                    }
                })
            })
            .collect();

        for handle in handles {
            handle.await.unwrap();
        }

        let duration = start.elapsed();
        println!("Concurrent performance test completed in {:?}", duration);
        
        // Assert performance meets requirements
        assert!(duration < Duration::from_secs(30));
    }

    #[tokio::test]
    async fn test_query_performance() {
        let db = setup_performance_test().await;
        let start = Instant::now();
        
        // Perform complex query operations
        for _ in 0..1000 {
            db.client
                .query("SELECT * FROM test_table WHERE field1 = $value")
                .bind(("value", "test"))
                .await
                .unwrap();
        }

        let duration = start.elapsed();
        println!("Query performance test completed in {:?}", duration);
        
        // Assert query performance meets requirements
        assert!(duration < Duration::from_secs(5));
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::security::SecurityManager;
    use crate::sanitizer::Sanitizer;
    use crate::anomaly_detection::AnomalyDetector;

    #[test]
    fn test_sanitizer() {
        let sanitizer = Sanitizer::new();
        
        // Test valid input
        assert!(sanitizer.sanitize_input("valid_input123").is_ok());
        
        // Test invalid input
        assert!(sanitizer.sanitize_input("invalid;input").is_err());
    }

    #[test]
    fn test_security_validation() {
        let config = SecurityConfig {
            max_query_depth: 3,
            max_array_length: 100,
            blocked_keywords: vec!["DROP".to_string()],
        };
        
        let client = // ... mock client setup
        let security = SecurityManager::new(client, config);
        
        // Test query depth validation
        let deep_query = "SELECT * FROM (SELECT * FROM (SELECT * FROM users))";
        assert!(security.validate_query(deep_query).is_err());
    }

    #[test]
    fn test_anomaly_detection() {
        let mut detector = AnomalyDetector::new(10, 2.0);
        
        // Add some normal metrics
        for _ in 0..5 {
            detector.record_metrics(QueryMetrics {
                execution_time: Duration::from_millis(100),
                rows_affected: 10,
                timestamp: Instant::now(),
            });
        }
        
        // Test anomaly detection
        let anomaly_metrics = QueryMetrics {
            execution_time: Duration::from_millis(1000),
            rows_affected: 100,
            timestamp: Instant::now(),
        };
        
        let anomalies = detector.detect_anomalies(&anomaly_metrics);
        assert!(!anomalies.is_empty());
    }
}
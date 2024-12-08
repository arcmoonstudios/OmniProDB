use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::time::{Duration, SystemTime};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryMetrics {
    pub execution_time: Duration,
    pub rows_affected: usize,
    pub timestamp: SystemTime,
}

pub struct AnomalyDetector {
    window_size: usize,
    metrics_history: VecDeque<QueryMetrics>,
    threshold_multiplier: f64,
}

impl AnomalyDetector {
    pub fn new(window_size: usize, threshold_multiplier: f64) -> Self {
        Self {
            window_size,
            metrics_history: VecDeque::with_capacity(window_size),
            threshold_multiplier,
        }
    }

    pub fn record_metrics(&mut self, metrics: QueryMetrics) {
        if self.metrics_history.len() >= self.window_size {
            self.metrics_history.pop_front();
        }
        self.metrics_history.push_back(metrics);
    }

    pub fn detect_anomalies(&self, current_metrics: &QueryMetrics) -> Vec<String> {
        let mut anomalies = Vec::new();
        
        if self.metrics_history.len() < 2 {
            return anomalies;
        }

        // Calculate average execution time
        let avg_execution_time: Duration = self.metrics_history
            .iter()
            .map(|m| m.execution_time)
            .sum::<Duration>() / self.metrics_history.len() as u32;

        // Check for execution time anomaly
        if current_metrics.execution_time > avg_execution_time.mul_f64(self.threshold_multiplier) {
            anomalies.push("Unusual query execution time detected".to_string());
        }

        // Check for unusual number of affected rows
        let avg_rows: f64 = self.metrics_history
            .iter()
            .map(|m| m.rows_affected as f64)
            .sum::<f64>() / self.metrics_history.len() as f64;

        if (current_metrics.rows_affected as f64) > avg_rows * self.threshold_multiplier {
            anomalies.push("Unusual number of affected rows detected".to_string());
        }

        anomalies
    }
}
//! Metrics and monitoring

use crate::Metrics;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Metrics collector
pub struct MetricsCollector {
    metrics: Arc<RwLock<Metrics>>,
}

impl MetricsCollector {
    /// Create new metrics collector
    pub fn new() -> Self {
        Self {
            metrics: Arc::new(RwLock::new(Metrics::default())),
        }
    }
    
    /// Get Prometheus-format metrics
    pub async fn prometheus_metrics(&self) -> String {
        let m = self.metrics.read().await;
        
        format!(
            "# HELP fisher_total_batches Total number of batches processed\n\
             # TYPE fisher_total_batches counter\n\
             fisher_total_batches {}\n\
             \n\
             # HELP fisher_total_intents Total number of intents processed\n\
             # TYPE fisher_total_intents counter\n\
             fisher_total_intents {}\n\
             \n\
             # HELP fisher_avg_savings_percent Average gas savings percentage\n\
             # TYPE fisher_avg_savings_percent gauge\n\
             fisher_avg_savings_percent {:.2}\n\
             \n\
             # HELP fisher_avg_batch_size Average batch size\n\
             # TYPE fisher_avg_batch_size gauge\n\
             fisher_avg_batch_size {:.2}\n",
            m.total_batches,
            m.total_intents,
            m.avg_savings_percent,
            m.avg_batch_size,
        )
    }
}

impl Default for MetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}

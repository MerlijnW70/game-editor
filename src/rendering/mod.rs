//! GPU-accelerated rendering systems and batching

pub mod batching_metrics_system;
pub mod batching_system;
pub mod gpu_batch;

pub use batching_metrics_system::{batching_input, collect_batching_metrics, log_batching_summary};
pub use batching_system::{BatchingMetrics, BatchingStrategy, BatchingSystem, SpriteBatch};
pub use gpu_batch::{BatchingStats, GpuBatchConfig, SpriteInstance};

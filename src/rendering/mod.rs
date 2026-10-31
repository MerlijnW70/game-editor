//! GPU-accelerated rendering systems and batching

pub mod batching_metrics_system;
pub mod batching_system;
pub mod gpu_batch;

pub use batching_system::{BatchingSystem, BatchingMetrics, BatchingStrategy, SpriteBatch};
pub use gpu_batch::{GpuBatchConfig, SpriteInstance, BatchingStats};
pub use batching_metrics_system::{collect_batching_metrics, log_batching_summary, batching_input};

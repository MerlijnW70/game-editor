//! Game resources: pooling, configuration, assets

pub mod pool_manager;
pub mod config;

pub use pool_manager::{PoolManager, PoolType, PoolStats};
pub use config::GameConfig;

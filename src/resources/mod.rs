//! Game resources: pooling, configuration, prefabs, assets

pub mod pool_manager;
pub mod config;
pub mod prefab;

pub use pool_manager::{PoolManager, PoolType, PoolStats};
pub use config::GameConfig;
pub use prefab::{PrefabLibrary, PrefabTemplate};

//! Game resources: pooling, configuration, prefabs, assets

pub mod config;
pub mod pool_manager;
pub mod prefab;

pub use config::GameConfig;
pub use pool_manager::{PoolManager, PoolStats, PoolType};
pub use prefab::{PrefabLibrary, PrefabTemplate};

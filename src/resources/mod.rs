//! Game resources: pooling, configuration, prefabs, assets

pub mod config;
pub mod pool_manager;
pub mod prefab;

pub use config::{ConfigError, ConfigFileError, ConfigPath, GameConfig};
pub use pool_manager::{PoolManager, PoolStats, PoolType, PoolTypeStats, Release};
pub use prefab::{PrefabLibrary, PrefabTemplate};

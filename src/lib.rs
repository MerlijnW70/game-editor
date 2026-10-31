//! Vampire Survivors-style 2D engine: core ECS and entity management.
//!
//! Module map (paths for other workers are intentionally left free):
//! - `components`: plain data components
//! - `archetypes`: bundle factories for player / enemy / projectile
//! - `systems`: movement, health, despawn, spawning, contact detection
//!   (`systems::rendering` and `systems::camera` are reserved)
//! - `collision`: spatial partitioning, collision detection, and response
//! - `pool`: memory pooling and entity recycling

pub mod archetypes;
pub mod bench;
pub mod collision;
pub mod components;
pub mod debug;
pub mod events;
pub mod game;
pub mod plugin;
pub mod rendering;
pub mod resources;
pub mod systems;

pub use plugin::{CoreEcsPlugin, CoreSet, RenderingPlugin, RenderSet};
pub use collision::{ActiveSpatialGrid, CollisionConfig, CollisionMessage};
pub use rendering::{GpuBatchConfig, SpriteInstance, BatchingStats};
pub use resources::{PoolManager, PoolType};

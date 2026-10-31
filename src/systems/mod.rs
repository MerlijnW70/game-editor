//! Core gameplay systems. `rendering` and `camera` modules are reserved for
//! the rendering worker and intentionally not declared here.

pub mod camera;
pub mod collision;
pub mod config;
pub mod contact;
pub mod despawn;
pub mod health;
pub mod movement;
pub mod pooling;
pub mod prefab_spawn;
pub mod rendering;
pub mod spawning;

pub use spawning::EnemySpawner;
pub use rendering::RenderMetrics;
pub use camera::spawn_camera;
pub use collision::CollisionDiagnostics;
pub use pooling::{pool_maintenance, log_pool_stats};
pub use config::{load_config, apply_config};
pub use prefab_spawn::{spawn_from_prefab, load_prefabs};

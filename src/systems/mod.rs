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
pub mod profiler_system;
pub mod rendering;
pub mod spawning;

pub use camera::spawn_camera;
pub use collision::CollisionDiagnostics;
pub use config::{apply_config, load_config};
pub use pooling::{log_pool_stats, pool_maintenance};
pub use prefab_spawn::{load_prefabs, spawn_from_prefab};
pub use profiler_system::{collect_profiler_metrics, log_profiler_summary, profiler_input};
pub use rendering::RenderMetrics;
pub use spawning::EnemySpawner;

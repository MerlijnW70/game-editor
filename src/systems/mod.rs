//! Core gameplay systems. `rendering` and `camera` modules are reserved for
//! the rendering worker and intentionally not declared here.

pub mod camera;
pub mod collision;
pub mod contact;
pub mod despawn;
pub mod health;
pub mod movement;
pub mod rendering;
pub mod spawning;

pub use spawning::EnemySpawner;
pub use rendering::RenderMetrics;
pub use camera::spawn_camera;
pub use collision::CollisionDiagnostics;

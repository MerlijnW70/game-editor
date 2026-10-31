//! Core gameplay systems. `rendering` and `camera` modules are reserved for
//! the rendering worker and intentionally not declared here.

pub mod contact;
pub mod despawn;
pub mod health;
pub mod movement;
pub mod spawning;

pub use spawning::EnemySpawner;

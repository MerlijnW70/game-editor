//! Vampire Survivors-style 2D engine: core ECS and entity management.
//!
//! Module map (paths for other workers are intentionally left free):
//! - `components`: plain data components
//! - `archetypes`: bundle factories for player / enemy / projectile
//! - `systems`: movement, health, despawn, spawning, contact detection
//!   (`systems::rendering` and `systems::camera` are reserved)
//! - `collision`: reserved for the collision worker

pub mod archetypes;
pub mod bench;
pub mod components;
pub mod events;
pub mod plugin;
pub mod systems;

pub use plugin::{CoreEcsPlugin, CoreSet};

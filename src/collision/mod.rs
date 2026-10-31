//! Collision detection and response system.
//!
//! - `spatial`: Grid-based spatial partitioning for efficient queries
//! - `detection`: Broad/narrow phase collision detection
//! - `response`: Collision response (knockback, separation)
//!
//! Integration: the collision worker uses this module and registers systems
//! that update the spatial grid and handle collisions each frame.

pub mod spatial;
pub mod detection;
pub mod response;

// Re-export main types
pub use spatial::{SpatialGrid, SpatialGridStats};
pub use detection::{ContactInfo, CollisionPair, test_circle_collision, test_circle_aabb_collision};
pub use response::{KnockbackConfig, CollisionResponse};

use bevy::prelude::*;

/// Resource: the active spatial grid. Updated by systems, queried during collision detection.
#[derive(Resource)]
pub struct ActiveSpatialGrid(pub SpatialGrid);

impl Default for ActiveSpatialGrid {
    fn default() -> Self {
        Self(SpatialGrid::new())
    }
}

/// Configuration for the collision system
#[derive(Resource, Clone)]
pub struct CollisionConfig {
    pub knockback_enabled: bool,
    pub knockback_force_scale: f32,
    pub team_filtering: bool,
    pub max_collisions_per_frame: usize,
}

impl Default for CollisionConfig {
    fn default() -> Self {
        Self {
            knockback_enabled: true,
            knockback_force_scale: 200.0,
            team_filtering: true,
            max_collisions_per_frame: 5000,
        }
    }
}

/// Message: collision detected between two entities
#[derive(Message, Debug, Clone, Copy)]
pub struct CollisionMessage {
    pub entity_a: Entity,
    pub entity_b: Entity,
    pub contact_point: Vec2,
    pub normal: Vec2,
    pub penetration: f32,
}

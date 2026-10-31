//! Collision detection and response system.
//!
//! - `spatial`: Grid-based spatial partitioning for efficient queries (O(n) worst-case)
//! - `quadtree`: Quadtree-based spatial partitioning (O(log n) queries, dynamic subdivision)
//! - `detection`: Broad/narrow phase collision detection
//! - `response`: Collision response (knockback, separation)
//!
//! Integration: the collision worker uses this module and registers systems
//! that update the spatial grid and handle collisions each frame.
//!
//! **Job #1 Note:** quadtree is a drop-in replacement for grid with better O(log n) performance.

pub mod detection;
pub mod quadtree;
pub mod response;
pub mod spatial;

// Re-export main types
pub use detection::{
    test_circle_aabb_collision, test_circle_collision, CollisionPair, ContactInfo,
};
pub use quadtree::{QuadtreeIndex, AABB};
pub use response::{CollisionResponse, KnockbackConfig};
pub use spatial::{SpatialGrid, SpatialGridStats};

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

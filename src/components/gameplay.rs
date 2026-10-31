use bevy::prelude::*;

/// Marker for the player entity.
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct Player;

/// Marker for enemy entities.
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct Enemy;

/// Marker for projectile entities.
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct Projectile;

/// Player input state, written by an input source and read by gameplay systems.
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct PlayerInput {
    /// Desired move direction (not necessarily normalized, clamped to length 1).
    pub direction: Vec2,
    pub speed: f32,
}

impl Default for PlayerInput {
    fn default() -> Self {
        Self {
            direction: Vec2::ZERO,
            speed: 200.0,
        }
    }
}

/// Simple chase AI: move toward the nearest player.
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct EnemyAi {
    pub speed: f32,
}

/// Remaining time in seconds before the entity expires.
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Lifetime {
    pub remaining: f32,
}

impl Lifetime {
    pub const fn new(seconds: f32) -> Self {
        Self { remaining: seconds }
    }

    pub fn is_expired(&self) -> bool {
        self.remaining <= 0.0
    }
}

/// Marker: entity is disabled (pooled, not active). Skip systems that update disabled entities.
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct Disabled;

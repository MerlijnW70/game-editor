use bevy::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CollisionType {
    Player,
    Enemy,
    Projectile,
    Obstacle,
}

/// Circle collider. Detection only; the collision module owns resolution.
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Collision {
    pub radius: f32,
    pub kind: CollisionType,
}

impl Collision {
    pub const fn new(radius: f32, kind: CollisionType) -> Self {
        Self { radius, kind }
    }
}

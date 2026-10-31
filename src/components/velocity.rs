use bevy::prelude::*;

/// Movement vector in world units per second.
#[derive(Component, Debug, Clone, Copy, PartialEq, Default)]
pub struct Velocity(pub Vec2);

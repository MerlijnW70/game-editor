//! Messages (buffered events) shared between systems.

use crate::components::DamageType;
use bevy::prelude::*;

/// Request to damage `target`. Consumed by `systems::health::apply_damage`.
#[derive(Message, Debug, Clone, Copy)]
pub struct DamageEvent {
    pub target: Entity,
    pub amount: f32,
    pub kind: DamageType,
    pub source: Option<Entity>,
}

/// Emitted once when an entity reaches zero health.
#[derive(Message, Debug, Clone, Copy)]
pub struct DeathEvent {
    pub entity: Entity,
}

/// Emitted when two hostile colliders overlap (detection only).
#[derive(Message, Debug, Clone, Copy)]
pub struct ContactEvent {
    pub a: Entity,
    pub b: Entity,
}

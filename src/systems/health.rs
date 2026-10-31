use crate::components::{self, *};
use crate::events::{DamageEvent, DeathEvent};
use bevy::prelude::*;

/// Applies queued `DamageEvent`s and emits one `DeathEvent` per entity that
/// crosses zero health. Team filtering is done by whoever emits the event.
pub fn apply_damage(
    mut damage_events: MessageReader<DamageEvent>,
    mut death_events: MessageWriter<DeathEvent>,
    mut query: Query<&mut Health>,
) {
    for event in damage_events.read() {
        let Ok(mut health) = query.get_mut(event.target) else {
            continue;
        };
        let was_alive = !health.is_dead();
        health.apply_damage(event.amount);
        if was_alive && health.is_dead() {
            death_events.write(DeathEvent {
                entity: event.target,
            });
        }
    }
}

/// Switches sprite animation to `Dead` for entities that just died.
pub fn mark_dead_sprites(
    mut death_events: MessageReader<DeathEvent>,
    mut sprites: Query<&mut components::Sprite>,
) {
    for event in death_events.read() {
        if let Ok(mut sprite) = sprites.get_mut(event.entity) {
            sprite.animation = AnimationState::Dead;
        }
    }
}

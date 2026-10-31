use crate::components::{self, *};
use crate::debug::LogThrottle;
use crate::events::{DamageEvent, DeathEvent};
use bevy::prelude::*;

/// Applies queued `DamageEvent`s and emits one `DeathEvent` per entity that
/// crosses zero health. Team filtering is done by whoever emits the event.
///
/// Only finite, positive amounts are applied. Anything else is a bug in the
/// emitter: it is dropped (so it can never kill or heal) and reported at most
/// once per second with a count.
pub fn apply_damage(
    mut damage_events: MessageReader<DamageEvent>,
    mut death_events: MessageWriter<DeathEvent>,
    mut query: Query<&mut Health>,
    time: Res<Time<Real>>,
    mut report: Local<LogThrottle>,
) {
    let mut rejected = 0u64;
    let mut example = None;

    for event in damage_events.read() {
        if !(event.amount.is_finite() && event.amount > 0.0) {
            rejected += 1;
            example.get_or_insert(*event);
            continue;
        }
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

    if let (Some(total), Some(event)) = (report.record(time.elapsed_secs_f64(), rejected), example)
    {
        eprintln!(
            "Warning: ignored {total} damage event(s) with a non-positive or non-finite amount \
             since the last report (e.g. {event:?})"
        );
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

use crate::components::*;
use bevy::prelude::*;

/// Counts down `Lifetime` components.
pub fn tick_lifetimes(time: Res<Time>, mut query: Query<&mut Lifetime>) {
    let dt = time.delta_secs();
    for mut lifetime in &mut query {
        lifetime.remaining -= dt;
    }
}

/// Despawns dead entities (health <= 0) and expired lifetimes.
pub fn despawn_dead(
    mut commands: Commands,
    dead: Query<(Entity, &Health)>,
    expired: Query<(Entity, &Lifetime)>,
) {
    for (entity, health) in &dead {
        if health.is_dead() {
            commands.entity(entity).despawn();
        }
    }
    for (entity, lifetime) in &expired {
        if lifetime.is_expired() {
            // try_despawn: the entity may already be queued above.
            commands.entity(entity).try_despawn();
        }
    }
}

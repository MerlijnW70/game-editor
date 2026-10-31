use crate::components::*;
use bevy::prelude::*;

/// Counts down `Lifetime` components.
pub fn tick_lifetimes(time: Res<Time>, mut query: Query<&mut Lifetime>) {
    let dt = time.delta_secs();
    for mut lifetime in &mut query {
        lifetime.remaining -= dt;
    }
}

/// Marks dead entities (health <= 0) and expired lifetimes as disabled.
/// Avoids despawning to prevent entity generation counter overflow at scale.
pub fn despawn_dead(
    mut commands: Commands,
    dead: Query<(Entity, &Health), Without<Disabled>>,
    expired: Query<(Entity, &Lifetime), Without<Disabled>>,
) {
    for (entity, health) in &dead {
        if health.is_dead() {
            commands.entity(entity)
                .insert(Disabled)
                .insert(Visibility::Hidden);
        }
    }
    for (entity, lifetime) in &expired {
        if lifetime.is_expired() {
            commands.entity(entity)
                .insert(Disabled)
                .insert(Visibility::Hidden);
        }
    }
}

use crate::components::*;
use crate::resources::{PoolManager, PoolType};
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
/// Releases entities back to appropriate pools for reuse.
pub fn despawn_dead(
    mut commands: Commands,
    mut pool: ResMut<PoolManager>,
    dead_enemies: Query<(Entity, &Health), (With<Enemy>, Without<Disabled>)>,
    dead_projectiles: Query<(Entity, &Health), (With<Projectile>, Without<Disabled>)>,
    expired: Query<(Entity, &Lifetime), Without<Disabled>>,
) {
    // Handle dead enemies
    for (entity, health) in &dead_enemies {
        if health.is_dead() {
            commands.entity(entity)
                .insert(Disabled)
                .insert(Visibility::Hidden);
            pool.release(entity, PoolType::Enemy);
        }
    }

    // Handle dead projectiles
    for (entity, health) in &dead_projectiles {
        if health.is_dead() {
            commands.entity(entity)
                .insert(Disabled)
                .insert(Visibility::Hidden);
            pool.release(entity, PoolType::Projectile);
        }
    }

    // Handle expired lifetimes (effects)
    for (entity, lifetime) in &expired {
        if lifetime.is_expired() {
            commands.entity(entity)
                .insert(Disabled)
                .insert(Visibility::Hidden);
            pool.release(entity, PoolType::Effect);
        }
    }
}

use crate::components::*;
use crate::resources::{PoolManager, PoolType, Release};
use bevy::prelude::*;
use std::collections::HashSet;

/// Counts down `Lifetime` components of active entities.
pub fn tick_lifetimes(time: Res<Time>, mut query: Query<&mut Lifetime, Without<Disabled>>) {
    let dt = time.delta_secs();
    for mut lifetime in &mut query {
        lifetime.remaining -= dt;
    }
}

/// Filter: active (not pooled) entities carrying `T`.
type ActiveWith<T> = (With<T>, Without<Disabled>);
/// Lifetime plus the markers that decide which pool an expired entity joins.
type ExpiryData = (Entity, &'static Lifetime, Has<Enemy>, Has<Projectile>);

/// Retires dead enemies/projectiles and expired lifetimes.
///
/// Each retired entity goes back to the pool of its own type (projectiles to
/// the projectile pool, enemies to the enemy pool, anything else with a
/// `Lifetime` to the effect pool) and is disabled and hidden. When that pool
/// is full the entity is despawned instead; nothing is ever left disabled
/// outside a pool. An entity matched by several rules is retired once.
pub fn despawn_dead(
    mut commands: Commands,
    mut pool: ResMut<PoolManager>,
    dead_enemies: Query<(Entity, &Health), ActiveWith<Enemy>>,
    dead_projectiles: Query<(Entity, &Health), ActiveWith<Projectile>>,
    expired: Query<ExpiryData, Without<Disabled>>,
) {
    let mut retired = HashSet::new();
    let mut retire = |entity: Entity, pool_type: PoolType| {
        if !retired.insert(entity) {
            return;
        }
        match pool.release(entity, pool_type) {
            Release::Pooled => {
                commands
                    .entity(entity)
                    .insert((Disabled, Visibility::Hidden));
            }
            Release::Rejected => commands.entity(entity).despawn(),
        }
    };

    for (entity, health) in &dead_enemies {
        if health.is_dead() {
            retire(entity, PoolType::Enemy);
        }
    }
    for (entity, health) in &dead_projectiles {
        if health.is_dead() {
            retire(entity, PoolType::Projectile);
        }
    }
    for (entity, lifetime, is_enemy, is_projectile) in &expired {
        if lifetime.is_expired() {
            let pool_type = if is_projectile {
                PoolType::Projectile
            } else if is_enemy {
                PoolType::Enemy
            } else {
                PoolType::Effect
            };
            retire(entity, pool_type);
        }
    }
}

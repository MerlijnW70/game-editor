use crate::components::{
    Collision, Damage, Disabled, Enemy, EnemyAi, Health, Lifetime, Projectile, Sprite, Team,
    Velocity,
};
use crate::debug::LogThrottle;
use crate::resources::{PoolManager, PoolStats};
use bevy::prelude::*;

/// Strips gameplay components from entities the moment they are disabled, so
/// a pooled entity is inert even to systems that forget a `Without<Disabled>`
/// filter. Runs once per newly disabled entity (`Added<Disabled>`), not on
/// every pooled entity every frame. Reuse inserts a complete bundle.
pub fn pool_maintenance(mut commands: Commands, newly_disabled: Query<Entity, Added<Disabled>>) {
    for entity in &newly_disabled {
        commands.entity(entity).remove::<(
            Velocity,
            Health,
            Damage,
            Sprite,
            Enemy,
            Projectile,
            Lifetime,
            EnemyAi,
            Collision,
            Team,
        )>();
    }
}

/// Reports pool pressure on stderr: whenever misses (fresh spawns because a
/// pool was empty) or overflows (despawns because a pool was full) changed
/// since the last report, at most once per second.
pub fn log_pool_stats(
    pool: Res<PoolManager>,
    time: Res<Time<Real>>,
    mut last_reported: Local<PoolStats>,
    mut throttle: Local<LogThrottle>,
) {
    let stats = pool.stats();
    let pressure =
        |s: &PoolStats| [s.enemies, s.projectiles, s.effects].map(|p| (p.misses, p.overflows));
    if pressure(&stats) == pressure(&last_reported) {
        return;
    }
    if throttle.record(time.elapsed_secs_f64(), 1).is_none() {
        return;
    }
    eprintln!(
        "Pool stats | enemies {}/{} available ({} hits, {} misses, {} overflows) | \
         projectiles {}/{} ({} hits, {} misses, {} overflows) | \
         effects {}/{} ({} hits, {} misses, {} overflows)",
        stats.enemies.available,
        stats.enemies.capacity,
        stats.enemies.hits,
        stats.enemies.misses,
        stats.enemies.overflows,
        stats.projectiles.available,
        stats.projectiles.capacity,
        stats.projectiles.hits,
        stats.projectiles.misses,
        stats.projectiles.overflows,
        stats.effects.available,
        stats.effects.capacity,
        stats.effects.hits,
        stats.effects.misses,
        stats.effects.overflows,
    );
    *last_reported = stats;
}

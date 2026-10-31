use bevy::prelude::*;
use crate::components::{Disabled, Velocity, Health, Damage, Sprite, Enemy, Projectile, Lifetime};
use crate::resources::PoolManager;

/// System to reset and maintain entities in the pool
/// Clears components from pooled entities to prepare them for reuse
pub fn pool_maintenance(
    mut commands: Commands,
    disabled_entities: Query<Entity, With<Disabled>>,
) {
    for entity in disabled_entities.iter() {
        // Reset components on pooled entities
        commands.entity(entity)
            .remove::<Velocity>()
            .remove::<Health>()
            .remove::<Damage>()
            .remove::<Sprite>()
            .remove::<Enemy>()
            .remove::<Projectile>()
            .remove::<Lifetime>();
    }
}

/// Log pool statistics (for diagnostics)
pub fn log_pool_stats(pool: Res<PoolManager>) {
    let stats = pool.stats();
    if stats.total_in_use() > 0 && stats.total_in_use() % 100 == 0 {
        eprintln!(
            "Pool stats | Enemies: {}/{} | Projectiles: {}/{} | Effects: {}/{} | Total in use: {}",
            stats.enemies_in_use(),
            stats.enemies_total,
            stats.projectiles_in_use(),
            stats.projectiles_total,
            stats.effects_in_use(),
            stats.effects_total,
            stats.total_in_use(),
        );
    }
}

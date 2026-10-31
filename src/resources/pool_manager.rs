use bevy::prelude::*;
use std::collections::{HashSet, VecDeque};

/// Pool type enumeration for different entity categories
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PoolType {
    Enemy,
    Projectile,
    Effect,
}

/// Outcome of `PoolManager::release`.
#[must_use = "a rejected entity is not pooled: despawn it, or it stays in the world forever"]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Release {
    /// The entity is now in the pool (or already was) and will be reused.
    Pooled,
    /// The pool is full. The caller must despawn the entity.
    Rejected,
}

/// Counters for one pool. All counts are lifetime totals.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct PoolTypeStats {
    /// Entities waiting in the pool, ready for reuse.
    pub available: usize,
    /// Most entities the pool will hold.
    pub capacity: usize,
    /// `acquire` calls served from the pool.
    pub hits: u64,
    /// `acquire` calls that found the pool empty (the caller spawned fresh).
    pub misses: u64,
    /// `release` calls rejected because the pool was full (the caller despawned).
    pub overflows: u64,
}

/// One bounded pool of disabled entities.
#[derive(Debug, Default)]
struct Pool {
    free: VecDeque<Entity>,
    members: HashSet<Entity>,
    capacity: usize,
    hits: u64,
    misses: u64,
    overflows: u64,
}

impl Pool {
    fn with_capacity(capacity: usize) -> Self {
        Self {
            free: VecDeque::with_capacity(capacity),
            members: HashSet::with_capacity(capacity),
            capacity,
            ..Default::default()
        }
    }

    fn acquire(&mut self) -> Option<Entity> {
        match self.free.pop_front() {
            Some(entity) => {
                self.members.remove(&entity);
                self.hits += 1;
                Some(entity)
            }
            None => {
                self.misses += 1;
                None
            }
        }
    }

    fn release(&mut self, entity: Entity) -> Release {
        if self.members.contains(&entity) {
            // Releasing twice must not let two spawns share one entity.
            return Release::Pooled;
        }
        if self.free.len() >= self.capacity {
            self.overflows += 1;
            return Release::Rejected;
        }
        self.free.push_back(entity);
        self.members.insert(entity);
        Release::Pooled
    }

    fn stats(&self) -> PoolTypeStats {
        PoolTypeStats {
            available: self.free.len(),
            capacity: self.capacity,
            hits: self.hits,
            misses: self.misses,
            overflows: self.overflows,
        }
    }
}

/// Bounded pools of disabled entities for high-churn entity types.
///
/// The pools only track entity IDs. Callers own the component state: disable
/// an entity before releasing it, despawn it when `release` returns
/// `Release::Rejected`, and insert a complete bundle after `acquire`.
#[derive(Resource, Debug)]
pub struct PoolManager {
    enemies: Pool,
    projectiles: Pool,
    effects: Pool,
}

impl Default for PoolManager {
    fn default() -> Self {
        Self::new(100, 500, 200)
    }
}

impl PoolManager {
    pub fn new(max_enemies: usize, max_projectiles: usize, max_effects: usize) -> Self {
        Self {
            enemies: Pool::with_capacity(max_enemies),
            projectiles: Pool::with_capacity(max_projectiles),
            effects: Pool::with_capacity(max_effects),
        }
    }

    fn pool_mut(&mut self, pool_type: PoolType) -> &mut Pool {
        match pool_type {
            PoolType::Enemy => &mut self.enemies,
            PoolType::Projectile => &mut self.projectiles,
            PoolType::Effect => &mut self.effects,
        }
    }

    /// Pre-fills every pool to capacity with hidden, disabled entities.
    pub fn initialize(&mut self, commands: &mut Commands) {
        for (pool_type, name) in [
            (PoolType::Enemy, "pooled_enemy"),
            (PoolType::Projectile, "pooled_projectile"),
            (PoolType::Effect, "pooled_effect"),
        ] {
            let pool = self.pool_mut(pool_type);
            while pool.free.len() < pool.capacity {
                let entity = commands
                    .spawn((
                        Name::new(name),
                        Visibility::Hidden,
                        crate::components::Disabled,
                    ))
                    .id();
                // Fresh entities are never already members, and there is room.
                let _ = pool.release(entity);
            }
        }
    }

    /// Takes a pooled entity, or `None` when the pool is empty (counted as a miss).
    pub fn acquire(&mut self, pool_type: PoolType) -> Option<Entity> {
        self.pool_mut(pool_type).acquire()
    }

    /// Returns a disabled entity to its pool. See `Release`.
    pub fn release(&mut self, entity: Entity, pool_type: PoolType) -> Release {
        self.pool_mut(pool_type).release(entity)
    }

    /// Snapshot of every pool's counters.
    pub fn stats(&self) -> PoolStats {
        PoolStats {
            enemies: self.enemies.stats(),
            projectiles: self.projectiles.stats(),
            effects: self.effects.stats(),
        }
    }
}

/// Counters for all pools.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct PoolStats {
    pub enemies: PoolTypeStats,
    pub projectiles: PoolTypeStats,
    pub effects: PoolTypeStats,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entity(index: u32) -> Entity {
        Entity::from_raw_u32(index).expect("test entity index is valid")
    }

    #[test]
    fn new_pools_are_empty_with_given_capacity() {
        let stats = PoolManager::new(50, 100, 25).stats();
        assert_eq!(stats.enemies.capacity, 50);
        assert_eq!(stats.projectiles.capacity, 100);
        assert_eq!(stats.effects.capacity, 25);
        assert_eq!(stats.enemies.available, 0);
    }

    #[test]
    fn acquire_counts_hits_and_misses() {
        let mut pool = PoolManager::new(5, 5, 5);
        assert_eq!(pool.acquire(PoolType::Enemy), None);
        assert_eq!(pool.release(entity(1), PoolType::Enemy), Release::Pooled);
        assert_eq!(pool.acquire(PoolType::Enemy), Some(entity(1)));
        let stats = pool.stats().enemies;
        assert_eq!((stats.hits, stats.misses, stats.available), (1, 1, 0));
        assert_eq!(
            pool.stats().projectiles,
            PoolTypeStats {
                capacity: 5,
                ..Default::default()
            }
        );
    }

    #[test]
    fn full_pool_rejects_and_counts_overflow() {
        let mut pool = PoolManager::new(2, 1, 1);
        assert_eq!(pool.release(entity(1), PoolType::Enemy), Release::Pooled);
        assert_eq!(pool.release(entity(2), PoolType::Enemy), Release::Pooled);
        assert_eq!(pool.release(entity(3), PoolType::Enemy), Release::Rejected);
        let stats = pool.stats().enemies;
        assert_eq!((stats.available, stats.overflows), (2, 1));
    }

    #[test]
    fn double_release_never_duplicates_an_entity() {
        let mut pool = PoolManager::new(5, 5, 5);
        assert_eq!(pool.release(entity(7), PoolType::Effect), Release::Pooled);
        assert_eq!(pool.release(entity(7), PoolType::Effect), Release::Pooled);
        assert_eq!(pool.stats().effects.available, 1);
        assert_eq!(pool.acquire(PoolType::Effect), Some(entity(7)));
        assert_eq!(pool.acquire(PoolType::Effect), None);
        // After being handed out, it may be released again.
        assert_eq!(pool.release(entity(7), PoolType::Effect), Release::Pooled);
    }

    #[test]
    fn pools_are_fifo_and_independent() {
        let mut pool = PoolManager::new(5, 5, 5);
        let _ = pool.release(entity(1), PoolType::Enemy);
        let _ = pool.release(entity(2), PoolType::Enemy);
        let _ = pool.release(entity(3), PoolType::Projectile);
        assert_eq!(pool.acquire(PoolType::Enemy), Some(entity(1)));
        assert_eq!(pool.acquire(PoolType::Enemy), Some(entity(2)));
        assert_eq!(pool.acquire(PoolType::Projectile), Some(entity(3)));
    }
}

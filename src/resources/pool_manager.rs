use bevy::prelude::*;
use std::collections::VecDeque;

/// Pool type enumeration for different entity categories
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PoolType {
    Enemy,
    Projectile,
    Effect,
}

/// Manages entity object pools for high-churn entity types
#[derive(Resource, Debug)]
pub struct PoolManager {
    enemy_pool: VecDeque<Entity>,
    projectile_pool: VecDeque<Entity>,
    effect_pool: VecDeque<Entity>,

    // Configuration
    max_enemies: usize,
    max_projectiles: usize,
    max_effects: usize,

    // Diagnostics
    pub total_acquired_enemies: usize,
    pub total_acquired_projectiles: usize,
    pub total_acquired_effects: usize,
}

impl Default for PoolManager {
    fn default() -> Self {
        Self {
            enemy_pool: VecDeque::with_capacity(100),
            projectile_pool: VecDeque::with_capacity(500),
            effect_pool: VecDeque::with_capacity(200),
            max_enemies: 100,
            max_projectiles: 500,
            max_effects: 200,
            total_acquired_enemies: 0,
            total_acquired_projectiles: 0,
            total_acquired_effects: 0,
        }
    }
}

impl PoolManager {
    pub fn new(max_enemies: usize, max_projectiles: usize, max_effects: usize) -> Self {
        Self {
            max_enemies,
            max_projectiles,
            max_effects,
            ..Default::default()
        }
    }

    /// Initialize pools by pre-allocating entity IDs
    pub fn initialize(&mut self, commands: &mut Commands) {
        // Pre-spawn enemies
        for _ in 0..self.max_enemies {
            let entity = commands
                .spawn((
                    Name::new("pooled_enemy"),
                    Transform::default(),
                    Visibility::Hidden,
                    crate::components::Disabled,
                ))
                .id();
            self.enemy_pool.push_back(entity);
        }

        // Pre-spawn projectiles
        for _ in 0..self.max_projectiles {
            let entity = commands
                .spawn((
                    Name::new("pooled_projectile"),
                    Transform::default(),
                    Visibility::Hidden,
                    crate::components::Disabled,
                ))
                .id();
            self.projectile_pool.push_back(entity);
        }

        // Pre-spawn effects
        for _ in 0..self.max_effects {
            let entity = commands
                .spawn((
                    Name::new("pooled_effect"),
                    Transform::default(),
                    Visibility::Hidden,
                    crate::components::Disabled,
                ))
                .id();
            self.effect_pool.push_back(entity);
        }
    }

    /// Acquire an entity from the pool
    pub fn acquire(&mut self, pool_type: PoolType) -> Option<Entity> {
        let entity = match pool_type {
            PoolType::Enemy => {
                self.total_acquired_enemies += 1;
                self.enemy_pool.pop_front()
            }
            PoolType::Projectile => {
                self.total_acquired_projectiles += 1;
                self.projectile_pool.pop_front()
            }
            PoolType::Effect => {
                self.total_acquired_effects += 1;
                self.effect_pool.pop_front()
            }
        };
        entity
    }

    /// Release an entity back to the pool
    pub fn release(&mut self, entity: Entity, pool_type: PoolType) {
        match pool_type {
            PoolType::Enemy => {
                if self.enemy_pool.len() < self.max_enemies {
                    self.enemy_pool.push_back(entity);
                }
            }
            PoolType::Projectile => {
                if self.projectile_pool.len() < self.max_projectiles {
                    self.projectile_pool.push_back(entity);
                }
            }
            PoolType::Effect => {
                if self.effect_pool.len() < self.max_effects {
                    self.effect_pool.push_back(entity);
                }
            }
        }
    }

    /// Get pool statistics
    pub fn stats(&self) -> PoolStats {
        PoolStats {
            enemies_available: self.enemy_pool.len(),
            enemies_total: self.max_enemies,
            projectiles_available: self.projectile_pool.len(),
            projectiles_total: self.max_projectiles,
            effects_available: self.effect_pool.len(),
            effects_total: self.max_effects,
        }
    }
}

#[derive(Debug, Clone)]
pub struct PoolStats {
    pub enemies_available: usize,
    pub enemies_total: usize,
    pub projectiles_available: usize,
    pub projectiles_total: usize,
    pub effects_available: usize,
    pub effects_total: usize,
}

impl PoolStats {
    pub fn enemies_in_use(&self) -> usize {
        self.enemies_total - self.enemies_available
    }

    pub fn projectiles_in_use(&self) -> usize {
        self.projectiles_total - self.projectiles_available
    }

    pub fn effects_in_use(&self) -> usize {
        self.effects_total - self.effects_available
    }

    pub fn total_in_use(&self) -> usize {
        self.enemies_in_use() + self.projectiles_in_use() + self.effects_in_use()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pool_initialization() {
        let pool = PoolManager::new(50, 100, 50);
        let stats = pool.stats();
        assert_eq!(stats.enemies_total, 50);
        assert_eq!(stats.projectiles_total, 100);
        assert_eq!(stats.effects_total, 50);
    }

    #[test]
    fn test_pool_stats() {
        let pool = PoolManager::new(10, 20, 10);
        let stats = pool.stats();
        assert_eq!(stats.enemies_in_use(), 0);
        assert_eq!(stats.projectiles_in_use(), 0);
        assert_eq!(stats.effects_in_use(), 0);
        assert_eq!(stats.total_in_use(), 0);
    }

    #[test]
    fn test_pool_diagnostics() {
        let mut pool = PoolManager::new(5, 10, 5);

        // Simulate acquiring entities
        pool.acquire(PoolType::Enemy);
        pool.acquire(PoolType::Enemy);
        pool.acquire(PoolType::Projectile);

        assert_eq!(pool.total_acquired_enemies, 2);
        assert_eq!(pool.total_acquired_projectiles, 1);
        assert_eq!(pool.total_acquired_effects, 0);
    }
}

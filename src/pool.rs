use bevy::prelude::*;
use crate::components::Disabled;
use std::collections::VecDeque;

/// Tracks pre-allocated entity pools for high-churn entity types.
#[derive(Resource, Debug)]
pub struct PoolManager {
    enemy_pool: VecDeque<Entity>,
    projectile_pool: VecDeque<Entity>,
    effect_pool: VecDeque<Entity>,
    max_enemies: usize,
    max_projectiles: usize,
    max_effects: usize,
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
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PoolType {
    Enemy,
    Projectile,
    Effect,
}

impl PoolManager {
    /// Initialize pools by pre-allocating entity IDs.
    pub fn initialize(&mut self, commands: &mut Commands) {
        // Pre-spawn enemy entities
        for _ in 0..self.max_enemies {
            let entity = commands.spawn((
                Name::new("pooled_enemy"),
                Transform::default(),
                Visibility::Hidden,
                Disabled,
            )).id();
            self.enemy_pool.push_back(entity);
        }

        // Pre-spawn projectile entities
        for _ in 0..self.max_projectiles {
            let entity = commands.spawn((
                Name::new("pooled_projectile"),
                Transform::default(),
                Visibility::Hidden,
                Disabled,
            )).id();
            self.projectile_pool.push_back(entity);
        }

        // Pre-spawn effect entities
        for _ in 0..self.max_effects {
            let entity = commands.spawn((
                Name::new("pooled_effect"),
                Transform::default(),
                Visibility::Hidden,
                Disabled,
            )).id();
            self.effect_pool.push_back(entity);
        }
    }

    /// Acquire an entity from the pool. Falls back to spawning if pool empty.
    pub fn acquire(&mut self, pool_type: PoolType, commands: &mut Commands) -> Entity {
        match pool_type {
            PoolType::Enemy => {
                if let Some(entity) = self.enemy_pool.pop_front() {
                    entity
                } else {
                    // Fallback: spawn if pool exhausted (shouldn't happen if sized correctly)
                    commands.spawn((
                        Name::new("overflow_enemy"),
                        Transform::default(),
                        Visibility::Inherited,
                    )).id()
                }
            }
            PoolType::Projectile => {
                if let Some(entity) = self.projectile_pool.pop_front() {
                    entity
                } else {
                    commands.spawn((
                        Name::new("overflow_projectile"),
                        Transform::default(),
                        Visibility::Inherited,
                    )).id()
                }
            }
            PoolType::Effect => {
                if let Some(entity) = self.effect_pool.pop_front() {
                    entity
                } else {
                    commands.spawn((
                        Name::new("overflow_effect"),
                        Transform::default(),
                        Visibility::Inherited,
                    )).id()
                }
            }
        }
    }

    /// Release entity back to pool. Must be called before despawn.
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

    /// Hide and disable entity for reuse. Minimal component manipulation to avoid generation churn.
    pub fn reset_pooled_entity(commands: &mut Commands, entity: Entity) {
        commands.entity(entity)
            .insert((Visibility::Hidden, Disabled))
            .insert(Transform::from_translation(Vec3::new(-10000.0, -10000.0, 0.0)));
    }
}

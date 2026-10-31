use crate::archetypes::enemy_bundle;
use crate::components::*;
use crate::resources::{PoolManager, PoolType};
use bevy::prelude::*;

/// Keeps the enemy population at `target_count` by spawning in batches on a
/// ring around the origin.
///
/// Pooling note: Bevy recycles entity indices (with generation bumps) and keeps
/// components packed in archetype tables, so churn does not fragment memory and
/// no manual pool is needed. Spawning goes through `spawn_batch`, which
/// reserves table capacity once per batch.
#[derive(Resource, Debug, Clone)]
pub struct EnemySpawner {
    pub target_count: usize,
    pub spawn_radius: f32,
    pub max_health: f32,
    pub speed: f32,
    /// Cap on spawns per frame to avoid hitches.
    pub max_per_frame: usize,
    rng: u64,
}

impl EnemySpawner {
    pub fn new(target_count: usize) -> Self {
        Self {
            target_count,
            spawn_radius: 400.0,
            max_health: 30.0,
            speed: 60.0,
            max_per_frame: 64,
            rng: 0x9E37_79B9_7F4A_7C15,
        }
    }

    /// xorshift64*, enough for scatter; deterministic across runs.
    fn next_f32(&mut self) -> f32 {
        self.rng ^= self.rng >> 12;
        self.rng ^= self.rng << 25;
        self.rng ^= self.rng >> 27;
        let v = self.rng.wrapping_mul(0x2545_F491_4F6C_DD1D);
        (v >> 40) as f32 / (1u64 << 24) as f32
    }
}

pub fn spawn_enemies(
    mut commands: Commands,
    mut spawner: ResMut<EnemySpawner>,
    mut pool: ResMut<PoolManager>,
    existing: Query<(), With<Enemy>>,
    all_entities: Query<()>,
) {
    // Cap total entities at 8192 to prevent generation overflow
    const MAX_ENTITIES: usize = 8192;
    if all_entities.iter().count() >= MAX_ENTITIES {
        return;
    }

    let missing = spawner.target_count.saturating_sub(existing.iter().len());
    let count = missing.min(spawner.max_per_frame);
    if count == 0 {
        return;
    }

    let (radius, health, speed) = (spawner.spawn_radius, spawner.max_health, spawner.speed);

    // Try to acquire entities from pool first
    let mut fallback_spawns = Vec::new();

    for _ in 0..count {
        if let Some(entity) = pool.acquire(PoolType::Enemy) {
            let angle = spawner.next_f32() * std::f32::consts::TAU;
            let r = radius * (0.75 + 0.25 * spawner.next_f32());
            let pos = Vec2::from_angle(angle) * r;

            // Configure the pooled entity with enemy components
            commands
                .entity(entity)
                .insert((
                    Health::new(health),
                    Velocity(Vec2::ZERO),
                    Enemy,
                    Transform2D {
                        position: pos,
                        ..Default::default()
                    },
                    Layer::default(),
                ))
                .remove::<Disabled>()
                .remove::<Visibility>();
        } else {
            // Pool exhausted, fall back to direct spawn
            let angle = spawner.next_f32() * std::f32::consts::TAU;
            let r = radius * (0.75 + 0.25 * spawner.next_f32());
            fallback_spawns.push(enemy_bundle(Vec2::from_angle(angle) * r, health, speed));
        }
    }

    // Spawn any fallback entities that couldn't use the pool
    if !fallback_spawns.is_empty() {
        commands.spawn_batch(fallback_spawns);
    }
}

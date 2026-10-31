use crate::archetypes::enemy_bundle;
use crate::components::*;
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
    existing: Query<(), With<Enemy>>,
) {
    let missing = spawner.target_count.saturating_sub(existing.iter().len());
    let count = missing.min(spawner.max_per_frame);
    if count == 0 {
        return;
    }
    let (radius, health, speed) = (spawner.spawn_radius, spawner.max_health, spawner.speed);
    let bundles: Vec<_> = (0..count)
        .map(|_| {
            let angle = spawner.next_f32() * std::f32::consts::TAU;
            let r = radius * (0.75 + 0.25 * spawner.next_f32());
            enemy_bundle(Vec2::from_angle(angle) * r, health, speed)
        })
        .collect();
    commands.spawn_batch(bundles);
}

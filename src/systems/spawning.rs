use crate::archetypes::enemy_bundle;
use crate::components::*;
use crate::debug::LogThrottle;
use crate::resources::{GameConfig, PoolManager, PoolType};
use bevy::prelude::*;

/// Keeps the enemy population at `target_count` by spawning on a ring around
/// the origin. Dead enemies are pooled by `despawn_dead` and reused here
/// first; new entities are created with `spawn_batch`, which reserves table
/// capacity once per batch.
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

/// Tops the live enemy population up towards `EnemySpawner::target_count`,
/// at most `max_per_frame` per frame.
///
/// - Pooled (disabled) enemies are reused first and get a complete fresh
///   `enemy_bundle`, so a recycled enemy is indistinguishable from a new one
///   (sprite, contact damage and current wave speed included).
/// - New entities are only created while the world holds fewer than
///   `GameConfig::game.max_entities`; reuse never counts against the cap.
///   Reaching the cap is reported at most once per second.
/// - Counts use archetype-level query lengths (`iter().len()` over `With` /
///   `Without` filters), which sum table sizes instead of visiting entities.
#[allow(clippy::too_many_arguments)]
pub fn spawn_enemies(
    mut commands: Commands,
    mut spawner: ResMut<EnemySpawner>,
    mut pool: ResMut<PoolManager>,
    config: Res<GameConfig>,
    live_enemies: Query<(), (With<Enemy>, Without<Disabled>)>,
    all_entities: Query<()>,
    time: Res<Time<Real>>,
    mut cap_report: Local<LogThrottle>,
) {
    let missing = spawner
        .target_count
        .saturating_sub(live_enemies.iter().len());
    let wanted = missing.min(spawner.max_per_frame);
    if wanted == 0 {
        return;
    }

    let (radius, health, speed) = (spawner.spawn_radius, spawner.max_health, spawner.speed);
    let mut room = config
        .game
        .max_entities
        .saturating_sub(all_entities.iter().len());
    let mut fresh = Vec::new();
    let mut capped = 0u64;

    for _ in 0..wanted {
        let angle = spawner.next_f32() * std::f32::consts::TAU;
        let r = radius * (0.75 + 0.25 * spawner.next_f32());
        let bundle = enemy_bundle(Vec2::from_angle(angle) * r, health, speed);

        if let Some(entity) = pool.acquire(PoolType::Enemy) {
            commands
                .entity(entity)
                .remove::<(Disabled, Visibility)>()
                .insert(bundle);
        } else if room > 0 {
            room -= 1;
            fresh.push(bundle);
        } else {
            capped += 1;
        }
    }

    if !fresh.is_empty() {
        commands.spawn_batch(fresh);
    }

    if let Some(total) = cap_report.record(time.elapsed_secs_f64(), capped) {
        eprintln!(
            "Warning: entity cap game.max_entities = {} reached; {total} enemy spawn(s) skipped \
             since the last report",
            config.game.max_entities
        );
    }
}

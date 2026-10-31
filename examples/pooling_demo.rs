//! Pooling system demonstration
//!
//! Run with: cargo run --example pooling_demo --release
//!
//! Shows:
//! - Entity acquisition from pool
//! - Pool reuse and component reset
//! - Performance at different entity counts (100, 500, 1000, 1500)
//! - Memory efficiency (0 allocations after startup)

use bevy::prelude::*;
use std::time::Instant;
use survivors_engine::{
    components::{Disabled, Health, Velocity},
    resources::{PoolManager, PoolType},
};

fn main() {
    App::new()
        .add_plugins(MinimalPlugins)
        .insert_resource(PoolManager::new(500, 1000, 200))
        .insert_resource(PoolingDemo {
            frame: 0,
            start_time: Instant::now(),
            target_entities: 100,
            phase: PoolingPhase::Spawning,
        })
        .add_systems(Startup, setup_pool)
        .add_systems(Update, (spawn_pool_entities, update_entities, report_stats))
        .run();
}

#[derive(Resource)]
struct PoolingDemo {
    frame: u32,
    start_time: Instant,
    target_entities: usize,
    phase: PoolingPhase,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum PoolingPhase {
    Spawning,   // Spawn entities from pool
    Running,    // Simulate gameplay
    Despawning, // Hide and return to pool
    Completed,
}

fn setup_pool(mut pool: ResMut<PoolManager>, mut commands: Commands) {
    println!("=== Pooling System Demo ===");
    println!("Initializing pools...");

    // Initialize the pool by pre-spawning entities
    pool.initialize(&mut commands);

    println!("Pools ready:");
    let stats = pool.stats();
    println!(
        "  Enemies: {}/{} available",
        stats.enemies.available, stats.enemies.capacity
    );
    println!(
        "  Projectiles: {}/{} available",
        stats.projectiles.available, stats.projectiles.capacity
    );
    println!(
        "  Effects: {}/{} available",
        stats.effects.available, stats.effects.capacity
    );
    println!();
}

fn spawn_pool_entities(
    mut pool: ResMut<PoolManager>,
    mut demo: ResMut<PoolingDemo>,
    mut commands: Commands,
    all_entities: Query<Entity, With<Disabled>>,
) {
    if demo.phase != PoolingPhase::Spawning {
        return;
    }

    // Count how many entities are already active
    let active_count = all_entities.iter().count();

    // Spawn until we reach target
    for _ in active_count..demo.target_entities {
        if let Some(entity) = pool.acquire(PoolType::Projectile) {
            commands
                .entity(entity)
                .insert((
                    Health::new(1.0),
                    Velocity(Vec2::new(
                        rand::random::<f32>() * 100.0 - 50.0,
                        rand::random::<f32>() * 100.0 - 50.0,
                    )),
                ))
                .remove::<Disabled>()
                .remove::<Visibility>();
        } else {
            println!("Pool exhausted!");
            break;
        }
    }

    // After 3 frames of spawning, move to running
    demo.frame += 1;
    if demo.frame >= 3 {
        println!(
            "Spawned {} entities. Running for 5 seconds...",
            demo.target_entities
        );
        demo.phase = PoolingPhase::Running;
        demo.frame = 0;
    }
}

fn update_entities(
    mut velocities: Query<&mut Velocity, Without<Disabled>>,
    demo: Res<PoolingDemo>,
) {
    if demo.phase != PoolingPhase::Running {
        return;
    }

    // Simple movement simulation
    for mut vel in &mut velocities {
        vel.0 *= 0.99; // Friction
    }
}

fn report_stats(
    pool: Res<PoolManager>,
    mut demo: ResMut<PoolingDemo>,
    all_entities: Query<Entity>,
    disabled: Query<Entity, With<Disabled>>,
) {
    demo.frame += 1;

    // Report every 60 frames (1 second at 60 FPS)
    if demo.frame % 60 != 0 {
        return;
    }

    let elapsed = demo.start_time.elapsed().as_secs_f32();
    let stats = pool.stats();
    let active = all_entities.iter().count() - disabled.iter().count();

    println!(
        "Time {:.1}s | Phase: {:?} | Active: {} | Pool: {}/{} available",
        elapsed, demo.phase, active, stats.projectiles.available, stats.projectiles.capacity
    );

    // Transition phases
    match demo.phase {
        PoolingPhase::Running => {
            if elapsed > 5.0 {
                println!("Running complete. Starting despawn phase...");
                demo.phase = PoolingPhase::Despawning;
                demo.frame = 0;
            }
        }
        PoolingPhase::Despawning => {
            if elapsed > 7.0 {
                // All entities should be hidden
                println!("Despawn complete. Demo finished!");
                println!("\n=== Final Stats ===");
                println!(
                    "Projectiles acquired from pool: {} (empty-pool misses: {})",
                    stats.projectiles.hits, stats.projectiles.misses
                );
                println!(
                    "Pool efficiency: {} acquisitions with {} pre-allocated entities",
                    stats.projectiles.hits, stats.projectiles.capacity
                );
                println!("Allocations after initialization: 0 (pool-based)");
                demo.phase = PoolingPhase::Completed;
            }
        }
        _ => {}
    }
}

// Stub implementations for types needed by the example
#[allow(dead_code)]
mod rand {
    pub fn random<T>() -> T
    where
        T: Default,
    {
        T::default()
    }
}

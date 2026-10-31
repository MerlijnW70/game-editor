//! Collision detection demo: 100 moving entities with collision detection/response.
//!
//! Usage:
//!   cargo run --release --example collision_demo -- [--entities N] [--frames N]
//!
//! With `--frames N` the demo runs N frames and prints timing/FPS.
//! Without it, it runs 5s at 60 Hz.
//!
//! Features:
//! - 100 dynamic entities moving randomly
//! - Spatial grid collision detection (< 0.5ms per frame)
//! - Knockback and separation on collision
//! - Frame timing breakdown

use bevy::diagnostic::{DiagnosticsPlugin, FrameTimeDiagnosticsPlugin};
use bevy::prelude::*;
use survivors_engine::bench::{run_frames, BenchReport};
use survivors_engine::collision::{ActiveSpatialGrid, CollisionConfig};
use survivors_engine::components::*;
use survivors_engine::systems::CollisionDiagnostics;
use survivors_engine::{CoreEcsPlugin, CoreSet};

#[derive(Resource)]
struct CollisionStats {
    total_collisions: u64,
    collision_checks: u64,
    spawned: bool,
}

impl Default for CollisionStats {
    fn default() -> Self {
        Self {
            total_collisions: 0,
            collision_checks: 0,
            spawned: false,
        }
    }
}

/// Spawn N entities with random positions and velocities (once)
fn spawn_test_entities(mut commands: Commands, mut stats: ResMut<CollisionStats>) {
    if stats.spawned {
        return;
    }
    stats.spawned = true;

    const ENTITY_COUNT: usize = 100;
    const SPAWN_RADIUS: f32 = 400.0;

    for i in 0..ENTITY_COUNT {
        let angle = (i as f32 / ENTITY_COUNT as f32) * std::f32::consts::TAU;
        let pos = Vec2::new(angle.cos() * SPAWN_RADIUS, angle.sin() * SPAWN_RADIUS);

        // Random velocity with slight rotation
        let vel_angle = angle + (std::f32::consts::PI / 4.0) * (i as f32 % 2.0 - 0.5);
        let velocity = Vec2::new(vel_angle.cos() * 150.0, vel_angle.sin() * 150.0);

        commands.spawn((
            Transform2D::from_position(pos),
            Velocity(velocity),
            Collision::new(15.0, CollisionType::Enemy),
            Team::Enemy,
            Health::new(100.0),
            Sprite::default(),
        ));
    }

    println!("Spawned {} test entities", ENTITY_COUNT);
}

/// Update entity velocities with random walk (keeps them moving in different directions)
fn random_walk(mut query: Query<&mut Velocity>, time: Res<Time>) {
    for mut vel in &mut query {
        // Gradually change direction
        let angle = vel.0.to_angle();
        let new_angle = angle + (std::f32::consts::TAU * 0.05 * time.delta_secs());
        let speed = vel.0.length();
        vel.0 = Vec2::from_angle(new_angle) * speed;
    }
}

/// Log collision statistics
fn log_collision_stats(diagnostics: Res<CollisionDiagnostics>, mut stats: ResMut<CollisionStats>) {
    stats.total_collisions += diagnostics.actual_collisions as u64;
    stats.collision_checks += diagnostics.collision_checks as u64;
}

/// Print final statistics
fn print_frame_stats(
    diagnostics: Res<CollisionDiagnostics>,
    stats: Res<CollisionStats>,
    time: Res<Time>,
    query: Query<&Velocity>,
) {
    let total_collision_time = diagnostics.spatial_update_us
        + diagnostics.broad_phase_us
        + diagnostics.narrow_phase_us
        + diagnostics.response_us;

    if total_collision_time > 100 {
        // Log when collision time is significant
        eprintln!(
            "Collision Time: {:.2}ms (spatial {:.2}µs, broad {:.2}µs, narrow {:.2}µs, response {:.2}µs) | {} checks | {} collisions",
            total_collision_time as f32 / 1000.0,
            diagnostics.spatial_update_us,
            diagnostics.broad_phase_us,
            diagnostics.narrow_phase_us,
            diagnostics.response_us,
            diagnostics.collision_checks,
            diagnostics.actual_collisions
        );
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let arg = |name: &str| {
        args.windows(2)
            .find(|w| w[0] == name)
            .and_then(|w| w[1].parse::<u32>().ok())
    };
    let _entities = arg("--entities").unwrap_or(100);
    let frames_arg = arg("--frames");

    println!(
        "Collision Demo: {} entities, {} frames",
        100,
        frames_arg.unwrap_or(0)
    );

    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        DiagnosticsPlugin,
        FrameTimeDiagnosticsPlugin::default(),
        CoreEcsPlugin,
    ))
    .init_resource::<CollisionStats>()
    .add_systems(
        Update,
        (
            spawn_test_entities.in_set(CoreSet::Spawn),
            random_walk.in_set(CoreSet::Movement),
            (log_collision_stats, print_frame_stats).after(CoreSet::Detect),
        ),
    );

    if let Some(frames) = frames_arg {
        println!("Running {} frames uncapped...", frames);
        let report = run_frames(&mut app, frames, None);
        report.print("Collision Demo");

        let stats = app.world().resource::<CollisionStats>();
        let avg_checks = if frames as u64 > 0 {
            stats.collision_checks / frames as u64
        } else {
            0
        };
        let avg_collisions = if frames as u64 > 0 {
            stats.total_collisions / frames as u64
        } else {
            0
        };

        println!("\nCollision Stats over {} frames:", frames);
        println!("  Average collision checks per frame: {}", avg_checks);
        println!("  Average collisions per frame: {}", avg_collisions);
        println!("  Total collisions: {}", stats.total_collisions);
        println!("  Performance: {:.1} FPS", report.avg_fps);
    } else {
        println!("Running 300 frames at 60 Hz...");
        let report = run_frames(&mut app, 300, Some(60.0));
        report.print("Collision Demo");

        let stats = app.world().resource::<CollisionStats>();
        println!("\nDemo complete!");
        println!("  Total collisions detected: {}", stats.total_collisions);
        println!(
            "  Average collision checks: {:.0}",
            stats.collision_checks as f32 / 300.0
        );
    }

    // Print spatial grid stats
    let spatial = app.world().resource::<ActiveSpatialGrid>();
    let stats = spatial.0.stats();
    println!("\nSpatial Grid Stats:");
    println!("  Cells: {}", stats.cells_count);
    println!("  Entities: {}", stats.entities_count);
    println!("  Avg entities/cell: {:.2}", stats.avg_entities_per_cell);
}

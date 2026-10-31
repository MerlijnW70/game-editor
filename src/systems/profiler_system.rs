use crate::components::Disabled;
use crate::debug::{DebugProfiler, FrameMetrics};
use crate::resources::PoolManager;
use crate::systems::collision::CollisionDiagnostics;
use bevy::prelude::*;

/// Collect frame metrics and update profiler.
///
/// Entity counts use archetype-level query lengths (`iter().len()` over
/// `With`/`Without` filters), which sum table sizes instead of visiting every
/// entity, so collecting metrics stays cheap at 8K+ entities.
pub fn collect_profiler_metrics(
    mut profiler: ResMut<DebugProfiler>,
    time: Res<Time>,
    pool: Res<PoolManager>,
    collisions: Res<CollisionDiagnostics>,
    all_entities: Query<()>,
    active_entities: Query<(), Without<Disabled>>,
) {
    if !profiler.enabled {
        return;
    }

    let delta_time = time.delta_secs();
    let fps = if delta_time > 0.0 {
        1.0 / delta_time
    } else {
        0.0
    };

    let pool_stats = pool.stats();
    let metrics = FrameMetrics {
        frame_number: profiler.current_frame.frame_number + 1,
        delta_time_ms: delta_time * 1000.0,
        fps,
        total_entities: all_entities.iter().len(),
        active_entities: active_entities.iter().len(),
        pool_enemies_available: pool_stats.enemies.available,
        pool_projectiles_available: pool_stats.projectiles.available,
        pool_effects_available: pool_stats.effects.available,
        spatial_queries_per_frame: collisions.spatial_queries as usize,
        collisions_detected: collisions.actual_collisions as usize,
    };

    profiler.record_frame(metrics);
}

/// Log profiler summary to console
pub fn log_profiler_summary(profiler: Res<DebugProfiler>, mut last_log: Local<u64>) {
    if !profiler.enabled {
        return;
    }

    // Log every 60 frames (1 second at 60 FPS)
    if profiler.current_frame.frame_number % 60 == 0
        && profiler.current_frame.frame_number != *last_log
    {
        let summary = profiler.get_summary();
        eprintln!("{}", summary.format_display());
        *last_log = profiler.current_frame.frame_number;
    }
}

/// Handle keyboard input for profiler toggle
pub fn profiler_input(input: Res<ButtonInput<KeyCode>>, mut profiler: ResMut<DebugProfiler>) {
    if input.just_pressed(KeyCode::KeyP) {
        profiler.toggle();
        eprintln!(
            "Profiler {}",
            if profiler.enabled {
                "ENABLED"
            } else {
                "DISABLED"
            }
        );
    }

    if input.just_pressed(KeyCode::KeyG) && profiler.enabled {
        profiler.toggle_graphs();
        eprintln!(
            "Graph view {}",
            if profiler.show_graphs { "ON" } else { "OFF" }
        );
    }
}

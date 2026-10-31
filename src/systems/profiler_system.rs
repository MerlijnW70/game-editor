use crate::debug::{DebugProfiler, FrameMetrics};
use crate::resources::PoolManager;
use bevy::prelude::*;

/// Collect frame metrics and update profiler
pub fn collect_profiler_metrics(
    mut profiler: ResMut<DebugProfiler>,
    time: Res<Time>,
    pool: Res<PoolManager>,
    all_entities: Query<()>,
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
        total_entities: all_entities.iter().count(),
        active_entities: all_entities.iter().count(), // Simplified: would need Disabled query
        pool_enemies_in_use: pool_stats.enemies_in_use(),
        pool_projectiles_in_use: pool_stats.projectiles_in_use(),
        pool_effects_in_use: pool_stats.effects_in_use(),
        spatial_queries_per_frame: 0, // Would be updated by collision system
        collisions_detected: 0,       // Would be updated by collision system
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

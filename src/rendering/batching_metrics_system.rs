use crate::rendering::BatchingSystem;
use bevy::prelude::*;

/// Collect batching metrics each frame
pub fn collect_batching_metrics(
    mut batching: ResMut<BatchingSystem>,
    query: Query<&Sprite, Without<crate::components::Player>>,
) {
    batching.metrics.reset();

    let sprite_count = query.iter().count();
    let batch_count = batching.calculate_batch_count(sprite_count);
    let batch_size = batching.max_batch_size;

    if batch_count > 0 {
        batching.metrics.record_batch(sprite_count);
        batching.metrics.finalize(batch_size);
    }
}

/// Log batching summary periodically
pub fn log_batching_summary(batching: Res<BatchingSystem>, mut frame_counter: Local<u64>) {
    *frame_counter += 1;

    // Log every 60 frames (approximately 1 second at 60 FPS)
    if *frame_counter % 60 == 0 && batching.enabled {
        eprintln!("{}", batching.format_summary());
        eprintln!(
            "  Sprites: {} | Batches: {} | Draw Calls: {:.1}% reduction",
            batching.metrics.total_sprites,
            batching.metrics.total_batches,
            batching.metrics.efficiency_percent()
        );
    }
}

/// Handle batching keyboard controls (B key)
pub fn batching_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut batching: ResMut<BatchingSystem>,
) {
    if keyboard_input.just_pressed(KeyCode::KeyB) {
        batching.toggle();
        eprintln!(
            "GPU Batching {}",
            if batching.enabled {
                "ENABLED"
            } else {
                "DISABLED"
            }
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_batching_metrics_reset() {
        let mut metrics = BatchingMetrics::default();
        metrics.total_sprites = 100;
        metrics.total_batches = 5;

        metrics.reset();
        assert_eq!(metrics.total_sprites, 0);
        assert_eq!(metrics.total_batches, 0);
    }
}

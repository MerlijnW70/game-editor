use crate::components::{AnimationFrames, Disabled, Layer, Sprite, Transform2D};
use bevy::prelude::*;
use std::collections::HashMap;

/// Performance metrics for rendering
#[derive(Resource, Debug, Default)]
pub struct RenderMetrics {
    pub draw_call_count: usize,
    pub total_sprites: usize,
    pub visible_sprites: usize,
    pub batches: usize,
    pub frame_time_ms: f32,
    pub fps: f32,
    pub frame_count: u32,
}

/// Marks entities that have been synced to Bevy rendering
#[derive(Component)]
pub struct RenderedEntity {
    pub bevy_entity: Entity,
}

/// Sync Transform2D and Sprite components to Bevy's rendering system
pub fn sync_sprites(
    active_query: Query<
        (
            Entity,
            &Transform2D,
            &Sprite,
            &Layer,
            Option<&AnimationFrames>,
        ),
        Without<Disabled>,
    >,
    total_query: Query<&Sprite>,
    mut metrics: ResMut<RenderMetrics>,
) {
    metrics.total_sprites = total_query.iter().len();
    metrics.visible_sprites = active_query.iter().len();
}

/// Update sprite animation frames
pub fn update_animations(
    mut query: Query<&mut AnimationFrames, Without<Disabled>>,
    time: Res<Time>,
) {
    for mut animation in &mut query {
        let _frame_changed = animation.update(time.delta_secs());
    }
}

/// Calculate and report batching metrics
pub fn calculate_metrics(
    sprites: Query<&Sprite>,
    mut metrics: ResMut<RenderMetrics>,
    time: Res<Time>,
) {
    let mut texture_counts: HashMap<String, usize> = HashMap::new();

    for sprite in &sprites {
        *texture_counts.entry(sprite.asset_path.clone()).or_insert(0) += 1;
    }

    // Each texture becomes one draw call (before GPU instancing)
    metrics.draw_call_count = texture_counts.len();
    metrics.batches = texture_counts.len();

    // Track frame time
    let delta = time.delta_secs();
    if delta > 0.0 {
        metrics.fps = 1.0 / delta;
        metrics.frame_time_ms = delta * 1000.0;
    }

    metrics.frame_count = metrics.frame_count.saturating_add(1);
}

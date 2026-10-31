use bevy::prelude::*;
use crate::components::{Sprite, Transform2D, Layer, AnimationFrames};
use std::collections::HashMap;

/// Performance metrics for rendering
#[derive(Resource, Debug, Default)]
pub struct RenderMetrics {
    pub draw_call_count: usize,
    pub total_sprites: usize,
    pub batches: usize,
}

/// Marks entities that have been synced to Bevy rendering
#[derive(Component)]
pub struct RenderedEntity {
    pub bevy_entity: Entity,
}

/// Sync Transform2D and Sprite components to Bevy's rendering system
pub fn sync_sprites(
    query: Query<(Entity, &Transform2D, &Sprite, &Layer, Option<&AnimationFrames>), Without<RenderedEntity>>,
    mut metrics: ResMut<RenderMetrics>,
) {
    // Count the sprites being rendered
    metrics.total_sprites = query.iter().len();
}

/// Update sprite animation frames
pub fn update_animations(
    mut query: Query<&mut AnimationFrames>,
    time: Res<Time>,
) {
    for mut animation in &mut query {
        let _frame_changed = animation.update(time.delta_secs());
        // Frame changed, update texture atlas if used
        // For now, we're using simple sprites, so this is a placeholder
        // In a full implementation, we'd update the texture atlas index here
    }
}

/// Calculate and report batching metrics
pub fn calculate_metrics(
    sprites: Query<&Sprite>,
    mut metrics: ResMut<RenderMetrics>,
) {
    let mut texture_counts: HashMap<String, usize> = HashMap::new();

    for sprite in &sprites {
        *texture_counts.entry(sprite.asset_path.clone()).or_insert(0) += 1;
    }

    // Each texture becomes one draw call in ideal batching
    metrics.draw_call_count = texture_counts.len();
    metrics.batches = texture_counts.len();
}

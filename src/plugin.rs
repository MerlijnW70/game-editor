use crate::collision::{ActiveSpatialGrid, CollisionConfig, CollisionMessage};
use crate::events::{ContactEvent, DamageEvent, DeathEvent};
use crate::systems::{camera, contact, despawn, health, movement, rendering, spawning, collision};
use bevy::prelude::*;

/// Ordered phases of the core simulation, so other plugins can schedule
/// relative to them (e.g. rendering after `CoreSet::Cleanup`).
#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CoreSet {
    Spawn,
    Ai,
    Movement,
    Detect,
    Health,
    Cleanup,
}

/// Phases for rendering, scheduled after core gameplay
#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RenderSet {
    SyncSprites,
    UpdateAnimations,
    SyncCamera,
    Cleanup,
}

/// Registers core ECS messages and systems. Insert an `EnemySpawner`
/// resource to enable automatic enemy population.
pub struct CoreEcsPlugin;

impl Plugin for CoreEcsPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<DamageEvent>()
            .add_message::<DeathEvent>()
            .add_message::<ContactEvent>()
            .add_message::<CollisionMessage>()
            .init_resource::<ActiveSpatialGrid>()
            .init_resource::<CollisionConfig>()
            .init_resource::<collision::CollisionDiagnostics>()
            .configure_sets(
                Update,
                (
                    CoreSet::Spawn,
                    CoreSet::Ai,
                    CoreSet::Movement,
                    CoreSet::Detect,
                    CoreSet::Health,
                    CoreSet::Cleanup,
                )
                    .chain(),
            )
            .add_systems(
                Update,
                (
                    spawning::spawn_enemies
                        .run_if(resource_exists::<spawning::EnemySpawner>)
                        .in_set(CoreSet::Spawn),
                    (movement::apply_player_input, movement::enemy_chase).in_set(CoreSet::Ai),
                    movement::movement.in_set(CoreSet::Movement),
                    (
                        collision::update_spatial_grid,
                        collision::detect_collisions,
                        collision::apply_collision_response,
                        collision::collisions_to_contact_events,
                        contact::contacts_to_damage,
                    )
                        .chain()
                        .in_set(CoreSet::Detect),
                    (health::apply_damage, health::mark_dead_sprites)
                        .chain()
                        .in_set(CoreSet::Health),
                    (despawn::tick_lifetimes, despawn::despawn_dead, collision::cleanup_spatial_grid_on_despawn)
                        .chain()
                        .in_set(CoreSet::Cleanup),
                ),
            );
    }
}

/// Rendering plugin: syncs ECS rendering components to Bevy's rendering system.
/// Requires DefaultPlugins or at least rendering plugins to be installed.
pub struct RenderingPlugin;

impl Plugin for RenderingPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<rendering::RenderMetrics>()
            .configure_sets(
                Update,
                (
                    RenderSet::SyncSprites,
                    RenderSet::UpdateAnimations,
                    RenderSet::SyncCamera,
                    RenderSet::Cleanup,
                )
                    .chain()
                    .after(CoreSet::Cleanup),
            )
            .add_systems(
                Update,
                (
                    rendering::sync_sprites.in_set(RenderSet::SyncSprites),
                    rendering::update_animations.in_set(RenderSet::UpdateAnimations),
                    camera::camera_follow_player.in_set(RenderSet::SyncCamera),
                    camera::sync_camera.in_set(RenderSet::SyncCamera),
                    rendering::calculate_metrics.in_set(RenderSet::Cleanup),
                ),
            );
    }
}

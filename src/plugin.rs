use crate::events::{ContactEvent, DamageEvent, DeathEvent};
use crate::systems::{contact, despawn, health, movement, spawning};
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

/// Registers core ECS messages and systems. Insert an `EnemySpawner`
/// resource to enable automatic enemy population.
pub struct CoreEcsPlugin;

impl Plugin for CoreEcsPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<DamageEvent>()
            .add_message::<DeathEvent>()
            .add_message::<ContactEvent>()
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
                    (contact::detect_contacts, contact::contacts_to_damage)
                        .chain()
                        .in_set(CoreSet::Detect),
                    (health::apply_damage, health::mark_dead_sprites)
                        .chain()
                        .in_set(CoreSet::Health),
                    (despawn::tick_lifetimes, despawn::despawn_dead)
                        .chain()
                        .in_set(CoreSet::Cleanup),
                ),
            );
    }
}

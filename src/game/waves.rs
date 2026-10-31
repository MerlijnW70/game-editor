//! Wave management and difficulty scaling.

use bevy::prelude::*;
use crate::systems::EnemySpawner;
use crate::game::state::GameState;

/// System to update wave spawning parameters based on current wave.
pub fn wave_system(
    game_state: Res<GameState>,
    mut spawner: ResMut<EnemySpawner>,
) {
    // Update spawner parameters based on current wave
    spawner.target_count = game_state.enemies_for_wave();
    spawner.max_health = game_state.enemy_max_health();
    spawner.speed = game_state.enemy_speed();

    // Increase spawn rate as waves progress (more spawns per frame)
    spawner.max_per_frame = (5 + game_state.current_wave as usize * 2).min(64);
}

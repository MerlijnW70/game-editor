//! Simple UI system: score, wave, health displays.
//!
//! In a headless build, this just logs to console.
//! In a rendered build, this would display text on screen.

use bevy::prelude::*;
use crate::game::state::GameState;
use crate::components::{Player, Health};

/// Resource to track UI state (for periodic logging in headless mode).
#[derive(Resource, Default)]
pub struct UiState {
    pub last_log_time: f32,
    pub log_interval: f32,
}

/// System to update and display UI information.
pub fn ui_system(
    game_state: Res<GameState>,
    player_query: Query<&Health, With<Player>>,
    mut ui_state: ResMut<UiState>,
    time: Res<Time>,
) {
    ui_state.last_log_time += time.delta_secs();

    if ui_state.last_log_time >= ui_state.log_interval {
        ui_state.last_log_time = 0.0;

        let player_health = match player_query.iter().next() {
            Some(h) => format!("{:.0}", h.current),
            None => "N/A".to_string(),
        };

        if game_state.is_game_over {
            println!(
                "GAME OVER: {} | Wave: {} | Score: {} | Reason: {}",
                player_health,
                game_state.current_wave,
                game_state.score,
                game_state.game_over_reason.as_deref().unwrap_or("Unknown")
            );
        } else {
            println!(
                "Health: {} | Wave: {} | Score: {} | Wave Progress: {:.1}%",
                player_health,
                game_state.current_wave,
                game_state.score,
                game_state.wave_progress * 100.0,
            );
        }
    }
}

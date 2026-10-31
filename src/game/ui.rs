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
            Some(h) => format!("{:.0}/{:.0}", h.current, h.max),
            None => "N/A".to_string(),
        };

        let streak_info = if game_state.current_kill_streak > 0 {
            format!(
                " | Streak: {}x ({:.1}x mult)",
                game_state.current_kill_streak,
                game_state.streak_multiplier()
            )
        } else {
            String::new()
        };

        let boss_marker = if game_state.is_boss_wave() {
            " [BOSS WAVE]"
        } else {
            ""
        };

        if game_state.is_game_over {
            println!(
                "\n╔═══════════════════════════════════════╗",
            );
            println!(
                "║ GAME OVER: {}",
                game_state.game_over_reason.as_deref().unwrap_or("Unknown")
            );
            println!(
                "║ Wave: {} | Score: {} | Time: {:.1}s",
                game_state.current_wave,
                game_state.score,
                game_state.time_survived
            );
            println!(
                "║ Best Streak: {}x | Health: {}",
                game_state.max_consecutive_kills,
                player_health
            );
            println!(
                "╚═══════════════════════════════════════╝\n",
            );
        } else {
            println!(
                "┃ Health: {} | Wave: {}{} | Score: {} | Progress: {:.1}% | Time: {:.1}s{}",
                player_health,
                game_state.current_wave,
                boss_marker,
                game_state.score,
                game_state.wave_progress * 100.0,
                game_state.time_survived,
                streak_info,
            );
        }
    }
}

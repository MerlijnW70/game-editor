//! Game state tracking: waves, score, game over, etc.

use bevy::prelude::*;

/// Tracks overall game state: current wave, score, whether the game is over.
#[derive(Resource, Debug, Clone)]
pub struct GameState {
    pub current_wave: u32,
    pub wave_progress: f32, // 0.0 to 1.0, time through current wave
    pub wave_duration: f32, // seconds
    pub score: u32,         // enemies killed
    pub is_game_over: bool,
    pub game_over_reason: Option<String>,
}

impl Default for GameState {
    fn default() -> Self {
        Self {
            current_wave: 1,
            wave_progress: 0.0,
            wave_duration: 30.0,
            score: 0,
            is_game_over: false,
            game_over_reason: None,
        }
    }
}

impl GameState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns enemy count for current wave (scales with wave number).
    pub fn enemies_for_wave(&self) -> usize {
        (20 + self.current_wave as usize * 10).min(200)
    }

    /// Returns max health for enemies in current wave.
    pub fn enemy_max_health(&self) -> f32 {
        30.0 + (self.current_wave as f32 - 1.0) * 5.0
    }

    /// Returns chase speed for enemies in current wave.
    pub fn enemy_speed(&self) -> f32 {
        60.0 + (self.current_wave as f32 - 1.0) * 10.0
    }

    /// Record a score increment (enemy killed).
    pub fn add_score(&mut self, amount: u32) {
        self.score += amount;
    }

    /// Check if wave should advance.
    pub fn should_advance_wave(&mut self, delta: f32) -> bool {
        self.wave_progress += delta / self.wave_duration;
        if self.wave_progress >= 1.0 {
            self.current_wave += 1;
            self.wave_progress = 0.0;
            true
        } else {
            false
        }
    }

    /// Mark game as over.
    pub fn end_game(&mut self, reason: impl Into<String>) {
        self.is_game_over = true;
        self.game_over_reason = Some(reason.into());
    }
}

/// System to update game state each frame and handle score tracking.
pub fn update_game_state(
    mut state: ResMut<GameState>,
    time: Res<Time>,
    player_health: Query<&crate::components::Health, With<crate::components::Player>>,
    mut death_events: bevy::prelude::MessageReader<crate::events::DeathEvent>,
    enemies: Query<&crate::components::Enemy>,
) {
    if state.is_game_over {
        return;
    }

    // Check if player is dead
    if let Some(health) = player_health.iter().next() {
        if health.is_dead() {
            state.end_game("Player defeated!");
            return;
        }
    }

    // Track score from enemy deaths
    for event in death_events.read() {
        // Check if the entity that died is an enemy
        if enemies.get(event.entity).is_ok() {
            state.add_score(1);
        }
    }

    // Advance wave progress
    let _advanced = state.should_advance_wave(time.delta_secs());
}

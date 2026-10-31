//! Game state tracking: waves, score, game over, etc.

use bevy::prelude::*;

/// Tracks overall game state: current wave, score, progression, game over.
#[derive(Resource, Debug, Clone)]
pub struct GameState {
    pub current_wave: u32,
    pub wave_progress: f32,     // 0.0 to 1.0, time through current wave
    pub wave_duration: f32,     // seconds per wave (decreases with difficulty)
    pub score: u32,             // enemies killed
    pub time_survived: f32,     // total time played in seconds
    pub max_consecutive_kills: u32, // combo multiplier
    pub current_kill_streak: u32,   // current combo counter
    pub difficulty_multiplier: f32, // scales with wave
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
            time_survived: 0.0,
            max_consecutive_kills: 0,
            current_kill_streak: 0,
            difficulty_multiplier: 1.0,
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

    /// Record a score increment and update kill streak.
    pub fn add_score(&mut self, amount: u32) {
        self.score += amount;
        self.current_kill_streak += amount;
        if self.current_kill_streak > self.max_consecutive_kills {
            self.max_consecutive_kills = self.current_kill_streak;
        }
    }

    /// Break the kill streak (called when player takes damage).
    pub fn break_streak(&mut self) {
        self.current_kill_streak = 0;
    }

    /// Get streak multiplier (1x to 5x based on current streak).
    pub fn streak_multiplier(&self) -> f32 {
        1.0 + (self.current_kill_streak as f32 / 50.0).min(4.0)
    }

    /// Check if wave should advance.
    pub fn should_advance_wave(&mut self, delta: f32) -> bool {
        self.wave_progress += delta / self.wave_duration;
        if self.wave_progress >= 1.0 {
            self.current_wave += 1;
            self.wave_progress = 0.0;
            // Increase difficulty: shorter waves, tougher enemies
            self.difficulty_multiplier = 1.0 + (self.current_wave as f32 * 0.1);
            // Cap wave duration at 15 seconds minimum
            self.wave_duration = (30.0 / self.difficulty_multiplier).max(15.0);
            true
        } else {
            false
        }
    }

    /// Update survival time and return elapsed delta.
    pub fn tick_time(&mut self, delta: f32) {
        self.time_survived += delta;
    }

    /// Mark game as over.
    pub fn end_game(&mut self, reason: impl Into<String>) {
        self.is_game_over = true;
        self.game_over_reason = Some(reason.into());
    }

    /// Get difficulty-adjusted enemy count for current wave.
    pub fn enemies_for_wave_adjusted(&self) -> usize {
        let base = (20 + self.current_wave as usize * 10).min(200);
        ((base as f32 * self.difficulty_multiplier).ceil() as usize).min(300)
    }

    /// Check if this wave is a boss wave (every 5th wave).
    pub fn is_boss_wave(&self) -> bool {
        self.current_wave % 5 == 0 && self.current_wave > 0
    }
}

/// System to update game state each frame: survival time, score, waves, game-over.
pub fn update_game_state(
    mut state: ResMut<GameState>,
    time: Res<Time>,
    player_health: Query<&crate::components::Health, With<crate::components::Player>>,
    mut damage_events: bevy::prelude::MessageReader<crate::events::DamageEvent>,
    mut death_events: bevy::prelude::MessageReader<crate::events::DeathEvent>,
    enemies: Query<&crate::components::Enemy>,
) {
    if state.is_game_over {
        return;
    }

    let delta = time.delta_secs();

    // Track survival time
    state.tick_time(delta);

    // Check if player is dead
    if let Some(health) = player_health.iter().next() {
        if health.is_dead() {
            state.end_game("Player defeated!");
            return;
        }
    }

    // Break streak if player takes damage
    let damage_events_count: usize = damage_events.read().count();
    if damage_events_count > 0 {
        state.break_streak();
    }

    // Track score from enemy deaths
    for event in death_events.read() {
        // Check if the entity that died is an enemy
        if enemies.get(event.entity).is_ok() {
            state.add_score(1);
        }
    }

    // Advance wave progress
    let _advanced = state.should_advance_wave(delta);
}

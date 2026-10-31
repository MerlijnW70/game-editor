use crate::resources::{ConfigPath, GameConfig, PoolManager};
use bevy::prelude::*;

/// Loads the game configuration from `ConfigPath` (default
/// `assets/config/game.yaml`) and creates the `PoolManager` from it.
///
/// Only a validated configuration is ever inserted: a missing, malformed or
/// invalid file falls back to `GameConfig::default()` with a stderr report.
pub fn load_config(mut commands: Commands, path: Option<Res<ConfigPath>>) {
    let path = path
        .map(|p| p.0.clone())
        .unwrap_or_else(|| ConfigPath::default().0);
    let config = GameConfig::load_or_default(&path);

    eprintln!(
        "Loaded config: {} ({} FPS target)",
        config.game.title, config.game.target_fps
    );
    eprintln!(
        "Pools: {} enemies, {} projectiles, {} effects",
        config.pools.max_enemies, config.pools.max_projectiles, config.pools.max_effects
    );
    eprintln!(
        "Enemies: {} HP, {} speed",
        config.enemies.base_health, config.enemies.base_speed
    );

    // Create PoolManager from config
    let pool = PoolManager::new(
        config.pools.max_enemies,
        config.pools.max_projectiles,
        config.pools.max_effects,
    );

    commands.insert_resource(config);
    commands.insert_resource(pool);
}

/// Apply configuration to game systems
pub fn apply_config(config: Res<GameConfig>) {
    // Log which systems have read the config
    if config.is_changed() {
        eprintln!("Game config updated - check individual systems for effects");
    }
}

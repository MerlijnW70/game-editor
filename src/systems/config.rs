use crate::resources::{GameConfig, PoolManager};
use bevy::prelude::*;

/// Load game configuration from assets/config/game.yaml
/// Creates PoolManager from config settings
pub fn load_config(mut commands: Commands) {
    let config = GameConfig::load_or_default("assets/config/game.yaml");

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

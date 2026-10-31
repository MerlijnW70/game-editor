use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

/// Game configuration loaded from YAML
#[derive(Resource, Debug, Clone, Serialize, Deserialize)]
pub struct GameConfig {
    pub game: GameSettings,
    pub pools: PoolConfig,
    pub enemies: EnemyConfig,
    pub waves: WaveConfig,
    pub rendering: RenderingConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameSettings {
    pub title: String,
    pub target_fps: u32,
    pub max_entities: usize,
    pub spawn_radius: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolConfig {
    pub max_enemies: usize,
    pub max_projectiles: usize,
    pub max_effects: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnemyConfig {
    pub base_health: f32,
    pub base_speed: f32,
    pub base_size: f32,
    pub spawn_interval: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WaveConfig {
    pub base_duration: f32,
    pub difficulty_scale: f32,
    pub boss_interval: u32,
    pub difficulty_multiplier: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenderingConfig {
    pub layer_count: u32,
    pub sprite_batch_size: usize,
    pub camera_smoothing: f32,
}

impl GameConfig {
    /// Load configuration from YAML file
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let config = serde_yaml::from_str(&content)?;
        Ok(config)
    }

    /// Load with fallback to defaults if file doesn't exist
    pub fn load_or_default<P: AsRef<Path>>(path: P) -> Self {
        Self::load(path).unwrap_or_else(|_| Self::default())
    }

    /// Save configuration to YAML file
    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<(), Box<dyn std::error::Error>> {
        let content = serde_yaml::to_string(self)?;
        fs::write(path, content)?;
        Ok(())
    }
}

impl Default for GameConfig {
    fn default() -> Self {
        Self {
            game: GameSettings {
                title: "Vampire Survivors".to_string(),
                target_fps: 60,
                max_entities: 8192,
                spawn_radius: 400.0,
            },
            pools: PoolConfig {
                max_enemies: 100,
                max_projectiles: 500,
                max_effects: 200,
            },
            enemies: EnemyConfig {
                base_health: 30.0,
                base_speed: 60.0,
                base_size: 10.0,
                spawn_interval: 0.5,
            },
            waves: WaveConfig {
                base_duration: 30.0,
                difficulty_scale: 0.1,
                boss_interval: 5,
                difficulty_multiplier: 1.0,
            },
            rendering: RenderingConfig {
                layer_count: 10,
                sprite_batch_size: 256,
                camera_smoothing: 0.1,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = GameConfig::default();
        assert_eq!(config.game.target_fps, 60);
        assert_eq!(config.pools.max_enemies, 100);
        assert_eq!(config.pools.max_projectiles, 500);
    }

    #[test]
    fn test_config_serialization() {
        let config = GameConfig::default();
        let yaml = serde_yaml::to_string(&config).unwrap();
        assert!(yaml.contains("target_fps"));
        assert!(yaml.contains("max_enemies"));
    }

    #[test]
    fn test_config_deserialization() {
        let yaml = r#"
game:
  title: "Test Game"
  target_fps: 60
  max_entities: 8192
  spawn_radius: 400.0
pools:
  max_enemies: 50
  max_projectiles: 250
  max_effects: 100
enemies:
  base_health: 20.0
  base_speed: 50.0
  base_size: 10.0
  spawn_interval: 1.0
waves:
  base_duration: 30.0
  difficulty_scale: 0.1
  boss_interval: 5
  difficulty_multiplier: 1.0
rendering:
  layer_count: 10
  sprite_batch_size: 256
  camera_smoothing: 0.1
"#;
        let config: GameConfig = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(config.pools.max_enemies, 50);
        assert_eq!(config.enemies.base_health, 20.0);
    }
}

use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// Where `systems::config::load_config` reads the game configuration from.
///
/// Defaults to `assets/config/game.yaml`. Insert a different value before
/// `Startup` runs to load another file (the integrated demo's `--config`).
#[derive(Resource, Debug, Clone, PartialEq, Eq)]
pub struct ConfigPath(pub PathBuf);

impl Default for ConfigPath {
    fn default() -> Self {
        Self(PathBuf::from("assets/config/game.yaml"))
    }
}

/// Game configuration loaded from YAML
#[derive(Resource, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GameConfig {
    pub game: GameSettings,
    pub pools: PoolConfig,
    pub enemies: EnemyConfig,
    pub waves: WaveConfig,
    pub rendering: RenderingConfig,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GameSettings {
    pub title: String,
    pub target_fps: u32,
    pub max_entities: usize,
    pub spawn_radius: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PoolConfig {
    pub max_enemies: usize,
    pub max_projectiles: usize,
    pub max_effects: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EnemyConfig {
    pub base_health: f32,
    pub base_speed: f32,
    pub base_size: f32,
    pub spawn_interval: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WaveConfig {
    pub base_duration: f32,
    pub difficulty_scale: f32,
    pub boss_interval: u32,
    pub difficulty_multiplier: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RenderingConfig {
    pub layer_count: u32,
    pub sprite_batch_size: usize,
    pub camera_smoothing: f32,
}

/// A configuration value that would break the simulation.
///
/// `GameConfig::validate` reports *every* failure: a single problem is
/// returned as its own variant, several as `MultipleFailed` (in field order).
/// Float rules also reject NaN and infinity, which plain `> 0.0` comparisons
/// would let through for `inf`.
#[derive(Debug, Clone, PartialEq)]
pub enum ConfigError {
    /// A pool capacity (`pools.<field>`) is zero.
    PoolSizeZero {
        /// Full YAML path of the offending field, e.g. `pools.max_enemies`.
        field: &'static str,
    },
    /// `game.max_entities` is zero.
    MaxEntitiesZero,
    /// `enemies.base_speed` is not a finite number greater than zero.
    EnemySpeedNotPositive {
        /// The rejected value.
        value: f32,
    },
    /// `enemies.spawn_interval` is not a finite number greater than zero.
    SpawnIntervalNotPositive {
        /// The rejected value.
        value: f32,
    },
    /// `game.spawn_radius` is negative or not finite.
    SpawnRadiusNegative {
        /// The rejected value.
        value: f32,
    },
    /// `rendering.sprite_batch_size` is zero.
    BatchSizeZero,
    /// A wave difficulty value (`waves.<field>`) is out of range.
    InvalidWaveDifficulty {
        /// Full YAML path of the offending field.
        field: &'static str,
        /// The rejected value.
        value: f32,
        /// The rule the value broke, e.g. `a finite number >= 0`.
        requirement: &'static str,
    },
    /// More than one value is invalid. Never nested and never empty.
    MultipleFailed(Vec<ConfigError>),
}

impl ConfigError {
    /// Every individual failure: the error itself, or the contents of
    /// `MultipleFailed`.
    pub fn failures(&self) -> &[ConfigError] {
        match self {
            ConfigError::MultipleFailed(errors) => errors,
            single => std::slice::from_ref(single),
        }
    }
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigError::PoolSizeZero { field } => write!(f, "{field} must be greater than 0"),
            ConfigError::MaxEntitiesZero => write!(f, "game.max_entities must be greater than 0"),
            ConfigError::EnemySpeedNotPositive { value } => write!(
                f,
                "enemies.base_speed must be a finite number greater than 0 (got {value})"
            ),
            ConfigError::SpawnIntervalNotPositive { value } => write!(
                f,
                "enemies.spawn_interval must be a finite number greater than 0 (got {value})"
            ),
            ConfigError::SpawnRadiusNegative { value } => write!(
                f,
                "game.spawn_radius must be a finite number >= 0 (got {value})"
            ),
            ConfigError::BatchSizeZero => {
                write!(f, "rendering.sprite_batch_size must be greater than 0")
            }
            ConfigError::InvalidWaveDifficulty {
                field,
                value,
                requirement,
            } => write!(f, "{field} must be {requirement} (got {value})"),
            ConfigError::MultipleFailed(errors) => {
                write!(f, "{} config values are invalid: ", errors.len())?;
                for (i, error) in errors.iter().enumerate() {
                    if i > 0 {
                        write!(f, "; ")?;
                    }
                    write!(f, "{error}")?;
                }
                Ok(())
            }
        }
    }
}

impl std::error::Error for ConfigError {}

/// Why a configuration file could not be loaded or saved.
#[derive(Debug)]
pub enum ConfigFileError {
    /// Reading or writing the file failed.
    Io(io::Error),
    /// The file is not valid YAML for `GameConfig`.
    Yaml(serde_yaml::Error),
    /// The file parsed, but its values are invalid.
    Invalid(ConfigError),
}

impl ConfigFileError {
    /// True when the file simply does not exist (a normal case for a fresh
    /// checkout), as opposed to an unreadable, malformed or invalid file.
    pub fn is_not_found(&self) -> bool {
        matches!(self, ConfigFileError::Io(e) if e.kind() == io::ErrorKind::NotFound)
    }
}

impl fmt::Display for ConfigFileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigFileError::Io(e) => write!(f, "I/O error: {e}"),
            ConfigFileError::Yaml(e) => write!(f, "invalid YAML: {e}"),
            ConfigFileError::Invalid(e) => write!(f, "invalid values: {e}"),
        }
    }
}

impl std::error::Error for ConfigFileError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ConfigFileError::Io(e) => Some(e),
            ConfigFileError::Yaml(e) => Some(e),
            ConfigFileError::Invalid(e) => Some(e),
        }
    }
}

impl From<io::Error> for ConfigFileError {
    fn from(e: io::Error) -> Self {
        ConfigFileError::Io(e)
    }
}

impl From<serde_yaml::Error> for ConfigFileError {
    fn from(e: serde_yaml::Error) -> Self {
        ConfigFileError::Yaml(e)
    }
}

impl From<ConfigError> for ConfigFileError {
    fn from(e: ConfigError) -> Self {
        ConfigFileError::Invalid(e)
    }
}

/// `value` is a real number strictly greater than zero.
fn finite_positive(value: f32) -> bool {
    value.is_finite() && value > 0.0
}

/// `value` is a real number greater than or equal to zero.
fn finite_non_negative(value: f32) -> bool {
    value.is_finite() && value >= 0.0
}

/// The stderr report for a `load_or_default` fallback. A missing file is a
/// plain note; anything else is a warning that names the reason.
fn fallback_message(path: &Path, error: &ConfigFileError) -> String {
    if error.is_not_found() {
        format!(
            "Config {} not found; using built-in defaults",
            path.display()
        )
    } else {
        format!(
            "Warning: config {} rejected ({error}); using built-in defaults",
            path.display()
        )
    }
}

impl GameConfig {
    /// Checks every value the simulation depends on and reports all failures.
    pub fn validate(&self) -> Result<(), ConfigError> {
        let mut errors = Vec::new();

        if self.game.max_entities == 0 {
            errors.push(ConfigError::MaxEntitiesZero);
        }
        if !finite_non_negative(self.game.spawn_radius) {
            errors.push(ConfigError::SpawnRadiusNegative {
                value: self.game.spawn_radius,
            });
        }
        for (field, capacity) in [
            ("pools.max_enemies", self.pools.max_enemies),
            ("pools.max_projectiles", self.pools.max_projectiles),
            ("pools.max_effects", self.pools.max_effects),
        ] {
            if capacity == 0 {
                errors.push(ConfigError::PoolSizeZero { field });
            }
        }
        if !finite_positive(self.enemies.base_speed) {
            errors.push(ConfigError::EnemySpeedNotPositive {
                value: self.enemies.base_speed,
            });
        }
        if !finite_positive(self.enemies.spawn_interval) {
            errors.push(ConfigError::SpawnIntervalNotPositive {
                value: self.enemies.spawn_interval,
            });
        }
        if !finite_non_negative(self.waves.difficulty_scale) {
            errors.push(ConfigError::InvalidWaveDifficulty {
                field: "waves.difficulty_scale",
                value: self.waves.difficulty_scale,
                requirement: "a finite number >= 0",
            });
        }
        if !finite_positive(self.waves.difficulty_multiplier) {
            errors.push(ConfigError::InvalidWaveDifficulty {
                field: "waves.difficulty_multiplier",
                value: self.waves.difficulty_multiplier,
                requirement: "a finite number greater than 0",
            });
        }
        if self.rendering.sprite_batch_size == 0 {
            errors.push(ConfigError::BatchSizeZero);
        }

        match errors.len() {
            0 => Ok(()),
            1 => Err(errors.remove(0)),
            _ => Err(ConfigError::MultipleFailed(errors)),
        }
    }

    /// Parses YAML and validates it. Only valid configurations are returned.
    pub fn from_yaml_str(yaml: &str) -> Result<Self, ConfigFileError> {
        let config: Self = serde_yaml::from_str(yaml)?;
        config.validate()?;
        Ok(config)
    }

    /// Loads and validates a configuration file.
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, ConfigFileError> {
        let content = fs::read_to_string(path)?;
        Self::from_yaml_str(&content)
    }

    /// Loads `path`, falling back to `GameConfig::default()` when the file is
    /// missing, unreadable, malformed or invalid. Every fallback is reported on
    /// stderr with the reason, so a broken config never passes silently.
    pub fn load_or_default<P: AsRef<Path>>(path: P) -> Self {
        let path = path.as_ref();
        match Self::load(path) {
            Ok(config) => config,
            Err(error) => {
                eprintln!("{}", fallback_message(path, &error));
                Self::default()
            }
        }
    }

    /// Validates, then writes the configuration as YAML. An invalid
    /// configuration is never written, so `save` cannot produce a file that
    /// `load` would reject.
    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<(), ConfigFileError> {
        self.validate()?;
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
    fn fallback_message_distinguishes_missing_from_rejected() {
        let path = Path::new("cfg.yaml");
        let missing = ConfigFileError::Io(io::Error::from(io::ErrorKind::NotFound));
        assert_eq!(
            fallback_message(path, &missing),
            "Config cfg.yaml not found; using built-in defaults"
        );

        let denied = ConfigFileError::Io(io::Error::from(io::ErrorKind::PermissionDenied));
        assert!(fallback_message(path, &denied)
            .starts_with("Warning: config cfg.yaml rejected (I/O error"));

        let invalid = ConfigFileError::Invalid(ConfigError::MaxEntitiesZero);
        assert_eq!(
            fallback_message(path, &invalid),
            "Warning: config cfg.yaml rejected (invalid values: game.max_entities must be greater than 0); using built-in defaults"
        );
    }

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

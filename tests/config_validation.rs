//! Job 01: `GameConfig` validation and fail-closed loading.
//!
//! Test names start with `config_validation_` so `cargo test config_validation`
//! selects exactly this suite.

use bevy::prelude::*;
use std::path::PathBuf;
use survivors_engine::resources::{
    ConfigError, ConfigFileError, ConfigPath, GameConfig, PoolManager,
};
use survivors_engine::CoreEcsPlugin;

/// A unique scratch path per test, removed on drop.
struct TempFile(PathBuf);

impl TempFile {
    fn new(name: &str) -> Self {
        let path = std::env::temp_dir().join(format!(
            "survivors_engine_{}_{}.yaml",
            std::process::id(),
            name
        ));
        let _ = std::fs::remove_file(&path);
        Self(path)
    }

    fn with_contents(name: &str, contents: &str) -> Self {
        let file = Self::new(name);
        std::fs::write(&file.0, contents).expect("temp dir is writable");
        file
    }
}

impl Drop for TempFile {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}

fn single_error(config: &GameConfig) -> ConfigError {
    let error = config.validate().expect_err("config should be invalid");
    assert_eq!(
        error.failures().len(),
        1,
        "expected exactly one failure, got {error}"
    );
    error
}

#[test]
fn config_validation_default_is_valid() {
    assert_eq!(GameConfig::default().validate(), Ok(()));
}

#[test]
fn config_validation_shipped_game_yaml_is_valid() {
    let path = ConfigPath::default().0;
    GameConfig::load(&path).unwrap_or_else(|e| panic!("{} must load: {e}", path.display()));
}

#[test]
fn config_validation_zero_pool_sizes_name_the_field() {
    type Breaker = fn(&mut GameConfig);
    let cases: [(&str, Breaker); 3] = [
        ("pools.max_enemies", |c| c.pools.max_enemies = 0),
        ("pools.max_projectiles", |c| c.pools.max_projectiles = 0),
        ("pools.max_effects", |c| c.pools.max_effects = 0),
    ];
    for (field, break_it) in cases {
        let mut config = GameConfig::default();
        break_it(&mut config);
        assert_eq!(single_error(&config), ConfigError::PoolSizeZero { field });
    }
}

#[test]
fn config_validation_zero_max_entities() {
    let mut config = GameConfig::default();
    config.game.max_entities = 0;
    assert_eq!(single_error(&config), ConfigError::MaxEntitiesZero);
}

#[test]
fn config_validation_zero_batch_size() {
    let mut config = GameConfig::default();
    config.rendering.sprite_batch_size = 0;
    assert_eq!(single_error(&config), ConfigError::BatchSizeZero);
}

#[test]
fn config_validation_enemy_speed_must_be_finite_positive() {
    for bad in [-10.0, 0.0, f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
        let mut config = GameConfig::default();
        config.enemies.base_speed = bad;
        assert!(
            matches!(
                single_error(&config),
                ConfigError::EnemySpeedNotPositive { .. }
            ),
            "base_speed {bad} must be rejected"
        );
    }
    let mut config = GameConfig::default();
    config.enemies.base_speed = f32::MIN_POSITIVE;
    assert_eq!(config.validate(), Ok(()));
}

#[test]
fn config_validation_spawn_interval_must_be_finite_positive() {
    for bad in [-0.5, 0.0, f32::NAN, f32::INFINITY] {
        let mut config = GameConfig::default();
        config.enemies.spawn_interval = bad;
        assert!(
            matches!(
                single_error(&config),
                ConfigError::SpawnIntervalNotPositive { .. }
            ),
            "spawn_interval {bad} must be rejected"
        );
    }
}

#[test]
fn config_validation_spawn_radius_allows_zero_rejects_negative_and_non_finite() {
    let mut config = GameConfig::default();
    config.game.spawn_radius = 0.0;
    assert_eq!(config.validate(), Ok(()), "zero radius is allowed");

    for bad in [-1.0, f32::NAN, f32::INFINITY] {
        let mut config = GameConfig::default();
        config.game.spawn_radius = bad;
        assert!(
            matches!(
                single_error(&config),
                ConfigError::SpawnRadiusNegative { .. }
            ),
            "spawn_radius {bad} must be rejected"
        );
    }
}

#[test]
fn config_validation_wave_difficulty_bounds() {
    let mut config = GameConfig::default();
    config.waves.difficulty_scale = 0.0;
    assert_eq!(config.validate(), Ok(()), "difficulty_scale 0 is allowed");

    for bad in [-0.1, f32::NAN, f32::INFINITY] {
        let mut config = GameConfig::default();
        config.waves.difficulty_scale = bad;
        assert!(matches!(
            single_error(&config),
            ConfigError::InvalidWaveDifficulty {
                field: "waves.difficulty_scale",
                ..
            }
        ));
    }
    for bad in [0.0, -1.0, f32::NAN, f32::INFINITY] {
        let mut config = GameConfig::default();
        config.waves.difficulty_multiplier = bad;
        assert!(matches!(
            single_error(&config),
            ConfigError::InvalidWaveDifficulty {
                field: "waves.difficulty_multiplier",
                ..
            }
        ));
    }
}

#[test]
fn config_validation_reports_every_failure_in_field_order() {
    let mut config = GameConfig::default();
    config.pools.max_enemies = 0;
    config.enemies.base_speed = -5.0;
    config.rendering.sprite_batch_size = 0;

    let error = config.validate().expect_err("three invalid values");
    assert_eq!(
        error,
        ConfigError::MultipleFailed(vec![
            ConfigError::PoolSizeZero {
                field: "pools.max_enemies"
            },
            ConfigError::EnemySpeedNotPositive { value: -5.0 },
            ConfigError::BatchSizeZero,
        ])
    );
    assert_eq!(error.failures().len(), 3);
}

#[test]
fn config_validation_messages_name_field_and_value() {
    let mut config = GameConfig::default();
    config.pools.max_projectiles = 0;
    config.enemies.base_speed = -5.0;
    let message = config.validate().expect_err("invalid").to_string();
    assert!(message.contains("2 config values are invalid"), "{message}");
    assert!(
        message.contains("pools.max_projectiles must be greater than 0"),
        "{message}"
    );
    assert!(message.contains("enemies.base_speed"), "{message}");
    assert!(message.contains("-5"), "{message}");
}

#[test]
fn config_validation_load_distinguishes_missing_malformed_and_invalid() {
    let missing = TempFile::new("missing");
    let error = GameConfig::load(&missing.0).expect_err("file does not exist");
    assert!(error.is_not_found(), "{error}");

    let malformed = TempFile::with_contents("malformed", "game: [this is not a config");
    let error = GameConfig::load(&malformed.0).expect_err("not valid YAML");
    assert!(matches!(error, ConfigFileError::Yaml(_)), "{error}");
    assert!(!error.is_not_found());

    let mut bad = GameConfig::default();
    bad.pools.max_enemies = 0;
    let invalid = TempFile::with_contents("invalid", &serde_yaml::to_string(&bad).unwrap());
    let error = GameConfig::load(&invalid.0).expect_err("values are invalid");
    assert!(
        matches!(
            &error,
            ConfigFileError::Invalid(ConfigError::PoolSizeZero { .. })
        ),
        "{error}"
    );
}

#[test]
fn config_validation_load_or_default_falls_back_without_panicking() {
    let mut bad = GameConfig::default();
    bad.game.title = "custom".into();
    bad.enemies.base_speed = f32::NAN;
    let invalid = TempFile::with_contents("fallback", &serde_yaml::to_string(&bad).unwrap());
    assert_eq!(
        GameConfig::load_or_default(&invalid.0),
        GameConfig::default()
    );

    let missing = TempFile::new("fallback_missing");
    assert_eq!(
        GameConfig::load_or_default(&missing.0),
        GameConfig::default()
    );
}

#[test]
fn config_validation_load_or_default_keeps_a_valid_file() {
    let mut custom = GameConfig::default();
    custom.pools.max_enemies = 7;
    let file = TempFile::with_contents("valid", &serde_yaml::to_string(&custom).unwrap());
    assert_eq!(GameConfig::load_or_default(&file.0), custom);
}

#[test]
fn config_validation_save_refuses_invalid_config() {
    let file = TempFile::new("save_invalid");
    let mut bad = GameConfig::default();
    bad.game.max_entities = 0;
    let error = bad
        .save(&file.0)
        .expect_err("invalid config must not be written");
    assert!(matches!(
        error,
        ConfigFileError::Invalid(ConfigError::MaxEntitiesZero)
    ));
    assert!(
        !file.0.exists(),
        "nothing may be written for an invalid config"
    );

    let good = TempFile::new("save_valid");
    GameConfig::default()
        .save(&good.0)
        .expect("valid config saves");
    assert_eq!(
        GameConfig::load(&good.0).expect("round-trips"),
        GameConfig::default()
    );
}

#[test]
fn config_validation_app_with_invalid_config_uses_defaults() {
    let mut bad = GameConfig::default();
    bad.pools.max_enemies = 0;
    bad.waves.difficulty_multiplier = -1.0;
    let file = TempFile::with_contents("app_invalid", &serde_yaml::to_string(&bad).unwrap());

    let mut app = App::new();
    app.add_plugins((MinimalPlugins, CoreEcsPlugin))
        .insert_resource(ConfigPath(file.0.clone()));
    app.update();

    assert_eq!(app.world().resource::<GameConfig>(), &GameConfig::default());
    let defaults = GameConfig::default();
    assert_eq!(
        app.world()
            .resource::<PoolManager>()
            .stats()
            .enemies
            .capacity,
        defaults.pools.max_enemies
    );
}

#[test]
fn config_validation_invalid_fixture_reports_every_bad_field() {
    let error =
        GameConfig::load("tests/fixtures/invalid_game.yaml").expect_err("fixture is invalid");
    let ConfigFileError::Invalid(error) = error else {
        panic!("fixture must parse and fail validation, got {error}");
    };
    let failures = error.failures();
    assert_eq!(failures.len(), 5, "{error}");
    assert_eq!(failures[0], ConfigError::MaxEntitiesZero);
    assert_eq!(
        failures[1],
        ConfigError::SpawnRadiusNegative { value: -50.0 }
    );
    assert_eq!(
        failures[2],
        ConfigError::PoolSizeZero {
            field: "pools.max_enemies"
        }
    );
    assert_eq!(
        failures[3],
        ConfigError::EnemySpeedNotPositive { value: -10.0 }
    );
    assert!(
        matches!(failures[4], ConfigError::SpawnIntervalNotPositive { value } if value.is_nan())
    );
}

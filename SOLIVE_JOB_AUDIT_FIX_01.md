# SOLIVE Job: Config Validation System

**Budget:** 0.12 SOL  
**Deadline:** 7 days  
**Effort:** 3-4 hours  
**Status:** Open for claims

---

## Spec

Implement comprehensive validation for `GameConfig` to catch invalid values before they cause runtime panics.

### Requirements

1. **Add `GameConfig::validate()` method** (src/resources/config.rs)
   ```rust
   pub fn validate(&self) -> Result<(), ConfigError>
   ```
   - Returns `ConfigError` enum with detailed field errors
   - Validates all numeric bounds and constraints

2. **ConfigError Enum** (src/resources/config.rs)
   ```rust
   #[derive(Debug)]
   pub enum ConfigError {
       PoolSizeZero,
       MaxEntitiesZero,
       EnemySpeedNegative,
       SpawnRadiusNegative,
       BatchSizeZero,
       InvalidWaveDifficulty,
       MultipleFailed(Vec<ConfigError>),
   }
   
   impl Display for ConfigError { /* ... */ }
   impl Error for ConfigError { /* ... */ }
   ```

3. **Validation Rules**
   - `pools.max_enemies > 0` (no zero-size pools)
   - `pools.max_projectiles > 0`
   - `pools.max_effects > 0`
   - `enemies.base_speed > 0.0` (positive speed)
   - `enemies.spawn_interval > 0.0` (positive spawn rate)
   - `game.spawn_radius >= 0.0` (non-negative)
   - `game.max_entities > 0`
   - `rendering.sprite_batch_size > 0` (no zero batch size)
   - `waves.difficulty_scale >= 0.0` (non-negative)
   - `waves.difficulty_multiplier > 0.0` (positive multiplier)

4. **Call validation in `GameConfig::load_or_default()`**
   ```rust
   pub fn load_or_default<P: AsRef<Path>>(path: P) -> Self {
       match Self::load(path) {
           Ok(config) => {
               match config.validate() {
                   Ok(_) => config,
                   Err(e) => {
                       eprintln!("Warning: Config validation failed: {}", e);
                       eprintln!("Using defaults instead");
                       Self::default()
                   }
               }
           }
           Err(_) => Self::default()
       }
   }
   ```

5. **Add tests** (tests/config_validation.rs)
   ```rust
   #[test]
   fn test_invalid_zero_pool_size() {
       let mut config = GameConfig::default();
       config.pools.max_enemies = 0;
       assert!(config.validate().is_err());
   }
   
   #[test]
   fn test_invalid_negative_speed() {
       let mut config = GameConfig::default();
       config.enemies.base_speed = -10.0;
       assert!(config.validate().is_err());
   }
   
   #[test]
   fn test_valid_config() {
       let config = GameConfig::default();
       assert!(config.validate().is_ok());
   }
   
   #[test]
   fn test_multiple_errors() {
       let mut config = GameConfig::default();
       config.pools.max_enemies = 0;
       config.enemies.base_speed = -5.0;
       assert!(config.validate().is_err());  // Multiple errors
   }
   ```

---

## Acceptance Criteria

✅ `ConfigError` enum defined with all 7 variants  
✅ `validate()` method checks all 10 constraints  
✅ Error messages are descriptive (field name + issue)  
✅ `load_or_default()` calls validate and logs warnings  
✅ All 4 test cases pass  
✅ No panics on invalid config  
✅ Compiles with no warnings  

---

## Verification Command

```bash
cargo test config_validation
cargo test --lib resources::config
cargo run --example integrated_demo -- --invalid-config  # Should not panic
```

---

## Judge Check

```bash
./verify_solive_delivery.ps1 JOB_CONFIG_VALIDATION
```

Expected: ✅ All constraints validated, no runtime panics on invalid config

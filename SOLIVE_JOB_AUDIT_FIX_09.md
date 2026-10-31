# SOLIVE Job: Centralized Config Validation Framework

**Budget:** 0.10 SOL  
**Deadline:** 7 days  
**Effort:** 3-4 hours  
**Status:** Phase 3 - Architecture  
**Depends On:** Job #8 (Custom Error Types)

---

## Problem Statement

Currently config validation is scattered:
- Some checks in GameConfig::load()
- Some in individual systems (spawning, rendering)
- Some missing entirely
- Hard to extend (add new check → modify multiple places)
- No standard validation framework

Production needs:
- Centralized validation (single source of truth)
- Extensible (add validators without touching core)
- Comprehensive (all constraints in one place)
- Reusable (same validators for load, reload, testing)

---

## Solution: Validator Framework

### 1. Validation Traits

```rust
use crate::GameError;

/// Core validation trait - implement for any config component
pub trait Validator {
    fn validate(&self) -> Result<(), GameError>;
}

/// Collector trait - implement for composite validations
pub trait ValidatorGroup {
    fn validate_all(&self) -> Result<(), Vec<GameError>>;
}

/// Validator combinator - for complex logic
pub struct ChainValidator {
    validators: Vec<Box<dyn Fn() -> Result<(), GameError>>>,
}

impl ChainValidator {
    pub fn new() -> Self {
        Self {
            validators: Vec::new(),
        }
    }
    
    pub fn add<F>(mut self, validator: F) -> Self
    where
        F: Fn() -> Result<(), GameError> + 'static,
    {
        self.validators.push(Box::new(validator));
        self
    }
    
    pub fn validate(&self) -> Result<(), Vec<GameError>> {
        let mut errors = Vec::new();
        
        for validator in &self.validators {
            if let Err(e) = validator() {
                errors.push(e);
            }
        }
        
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}
```

### 2. Domain Validators

```rust
// Pool Configuration Validator
pub struct PoolConfigValidator<'a>(&'a PoolConfig);

impl<'a> Validator for PoolConfigValidator<'a> {
    fn validate(&self) -> Result<(), GameError> {
        if self.0.max_enemies <= 0 {
            return Err(GameError::ConfigError {
                field: "pool.max_enemies".to_string(),
                reason: "must be positive".to_string(),
                value: self.0.max_enemies.to_string(),
            });
        }
        
        if self.0.max_projectiles <= 0 {
            return Err(GameError::ConfigError {
                field: "pool.max_projectiles".to_string(),
                reason: "must be positive".to_string(),
                value: self.0.max_projectiles.to_string(),
            });
        }
        
        Ok(())
    }
}

// Rendering Configuration Validator
pub struct RenderingConfigValidator<'a>(&'a RenderingConfig);

impl<'a> Validator for RenderingConfigValidator<'a> {
    fn validate(&self) -> Result<(), GameError> {
        // Batch size validation (from Job #7)
        if self.0.gpu_batch_size < 32 {
            return Err(GameError::ConfigError {
                field: "rendering.gpu_batch_size".to_string(),
                reason: "minimum is 32".to_string(),
                value: self.0.gpu_batch_size.to_string(),
            });
        }
        
        if self.0.gpu_batch_size > 1024 {
            return Err(GameError::ConfigError {
                field: "rendering.gpu_batch_size".to_string(),
                reason: "maximum is 1024".to_string(),
                value: self.0.gpu_batch_size.to_string(),
            });
        }
        
        Ok(())
    }
}

// Collision Configuration Validator
pub struct CollisionConfigValidator<'a>(&'a CollisionConfig);

impl<'a> Validator for CollisionConfigValidator<'a> {
    fn validate(&self) -> Result<(), GameError> {
        if self.0.grid_cell_size <= 0.0 {
            return Err(GameError::ConfigError {
                field: "collision.grid_cell_size".to_string(),
                reason: "must be positive".to_string(),
                value: self.0.grid_cell_size.to_string(),
            });
        }
        
        Ok(())
    }
}

// Spawning Configuration Validator
pub struct SpawningConfigValidator<'a>(&'a SpawningConfig);

impl<'a> Validator for SpawningConfigValidator<'a> {
    fn validate(&self) -> Result<(), GameError> {
        if self.0.initial_spawn_count == 0 {
            return Err(GameError::ConfigError {
                field: "spawning.initial_spawn_count".to_string(),
                reason: "must be at least 1".to_string(),
                value: "0".to_string(),
            });
        }
        
        if self.0.max_enemies <= self.0.initial_spawn_count {
            return Err(GameError::ConfigError {
                field: "spawning.max_enemies".to_string(),
                reason: "must be greater than initial_spawn_count".to_string(),
                value: self.0.max_enemies.to_string(),
            });
        }
        
        Ok(())
    }
}
```

### 3. Master GameConfig Validator

```rust
impl GameConfig {
    /// Comprehensive validation of entire config
    pub fn validate(&self) -> Result<(), Vec<GameError>> {
        let validators = ChainValidator::new()
            .add(|| PoolConfigValidator(&self.pool).validate())
            .add(|| RenderingConfigValidator(&self.rendering).validate())
            .add(|| CollisionConfigValidator(&self.collision).validate())
            .add(|| SpawningConfigValidator(&self.spawning).validate());
        
        validators.validate()
    }
    
    /// Load and validate from file
    pub fn load_and_validate(path: impl AsRef<std::path::Path>) -> Result<Self, Vec<GameError>> {
        let config = Self::load(path)?;
        config.validate()?;
        Ok(config)
    }
}
```

### 4. Validation Report

```rust
pub struct ValidationReport {
    pub valid: bool,
    pub errors: Vec<GameError>,
    pub warnings: Vec<String>,
}

impl ValidationReport {
    pub fn from_errors(errors: Vec<GameError>) -> Self {
        let valid = errors.is_empty();
        Self {
            valid,
            errors,
            warnings: Vec::new(),
        }
    }
    
    pub fn add_warning(&mut self, warning: String) {
        self.warnings.push(warning);
    }
    
    pub fn format_display(&self) -> String {
        let mut output = String::new();
        
        if self.valid {
            output.push_str("✅ Config validation passed\n");
        } else {
            output.push_str(&format!(
                "❌ Config validation failed ({} errors)\n",
                self.errors.len()
            ));
            
            for (i, error) in self.errors.iter().enumerate() {
                output.push_str(&format!("  {}. {}\n", i + 1, error));
            }
        }
        
        if !self.warnings.is_empty() {
            output.push_str(&format!("⚠️  Warnings ({}):\n", self.warnings.len()));
            for warning in &self.warnings {
                output.push_str(&format!("  • {}\n", warning));
            }
        }
        
        output
    }
}
```

### 5. System Integration

```rust
pub fn validate_config_on_load(
    config: Res<GameConfig>,
    mut events: EventWriter<ConfigValidationEvent>,
) {
    match config.validate() {
        Ok(()) => {
            info!("Config validation passed");
            events.send(ConfigValidationEvent::Valid);
        }
        Err(errors) => {
            error!("Config validation failed with {} errors", errors.len());
            for error in &errors {
                error!("  • {}", error);
            }
            events.send(ConfigValidationEvent::Invalid(errors));
        }
    }
}

#[derive(Event)]
pub enum ConfigValidationEvent {
    Valid,
    Invalid(Vec<GameError>),
}
```

---

## Acceptance Criteria

✅ `Validator` trait defined  
✅ `ValidatorGroup` trait for composition  
✅ `ChainValidator` for combining validators  
✅ Domain validators for 4+ config sections  
✅ `GameConfig::validate()` uses validators  
✅ `GameConfig::load_and_validate()` convenience  
✅ `ValidationReport` with formatted output  
✅ `ConfigValidationEvent` system  
✅ All 5 test cases pass  
✅ Compiles with no warnings  

---

## Tests

```rust
#[test]
fn test_pool_validator() {
    let valid_config = PoolConfig { max_enemies: 1000, max_projectiles: 500 };
    assert!(PoolConfigValidator(&valid_config).validate().is_ok());
    
    let invalid_config = PoolConfig { max_enemies: 0, max_projectiles: 500 };
    assert!(PoolConfigValidator(&invalid_config).validate().is_err());
}

#[test]
fn test_rendering_validator() {
    let valid = RenderingConfig { gpu_batch_size: 256 };
    assert!(RenderingConfigValidator(&valid).validate().is_ok());
    
    let too_small = RenderingConfig { gpu_batch_size: 16 };
    assert!(RenderingConfigValidator(&too_small).validate().is_err());
    
    let too_large = RenderingConfig { gpu_batch_size: 2048 };
    assert!(RenderingConfigValidator(&too_large).validate().is_err());
}

#[test]
fn test_chain_validator() {
    let validator = ChainValidator::new()
        .add(|| Ok(()))
        .add(|| Ok(()))
        .add(|| Err(GameError::GameLogicError {
            context: "test".to_string(),
            reason: "test error".to_string(),
        }));
    
    assert!(validator.validate().is_err());
}

#[test]
fn test_validation_report() {
    let errors = vec![
        GameError::ConfigError {
            field: "test1".to_string(),
            reason: "test".to_string(),
            value: "test".to_string(),
        },
    ];
    
    let report = ValidationReport::from_errors(errors);
    assert!(!report.valid);
    assert_eq!(report.errors.len(), 1);
    
    let display = report.format_display();
    assert!(display.contains("❌"));
}

#[test]
fn test_gameconfig_validate() {
    let config = GameConfig::default();
    // Should pass with default config
    assert!(config.validate().is_ok());
}
```

---

## Verification Command

```bash
cargo test test_pool_validator
cargo test test_rendering_validator
cargo test test_chain_validator
cargo test test_validation_report
cargo test test_gameconfig_validate
cargo run --release --example integrated_demo
# Should log validation results on startup
```

---

## Judge Check

```bash
./verify_solive_delivery.ps1 JOB_CONFIG_FRAMEWORK
```

Expected: ✅ Validators composable, GameConfig validates, all domains covered


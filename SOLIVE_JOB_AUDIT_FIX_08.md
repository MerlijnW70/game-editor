# SOLIVE Job: Custom Error Types & Recovery System

**Budget:** 0.08 SOL  
**Deadline:** 7 days  
**Effort:** 3-4 hours  
**Status:** Phase 3 - Architecture  
**Blocks:** Job #9 (Config Framework)

---

## Problem Statement

Current error handling is primitive:
- Panics on invalid data (crashes game)
- Silent failures (wrong behavior, no diagnostics)
- No error context (hard to debug)
- No recovery paths (no graceful degradation)

Production needs:
- Recoverable errors (continue gameplay)
- Detailed error context (debugging)
- Error categorization (handle by type)
- Fallback strategies (sensible defaults)

---

## Solution: Comprehensive Error Type System

### 1. Error Enum with Context

```rust
use std::fmt;

/// Comprehensive error type for game operations
#[derive(Debug, Clone)]
pub enum GameError {
    // Configuration errors
    ConfigError {
        field: String,
        reason: String,
        value: String,
    },
    
    // Pool errors
    PoolError {
        pool_type: String,
        reason: PoolErrorReason,
    },
    
    // Collision errors
    CollisionError {
        entity: u32,
        reason: CollisionErrorReason,
    },
    
    // Rendering errors
    RenderError {
        system: String,
        reason: String,
    },
    
    // Game logic errors
    GameLogicError {
        context: String,
        reason: String,
    },
    
    // IO errors
    IoError {
        path: String,
        error: std::io::ErrorKind,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PoolErrorReason {
    Exhausted,
    InvalidEntity,
    AlreadyReturned,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CollisionErrorReason {
    InvalidRadius,
    InvalidEntity,
    GridCorruption,
}

impl fmt::Display for GameError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            GameError::ConfigError { field, reason, value } => {
                write!(
                    f,
                    "Config error in field '{}': {} (value: {})",
                    field, reason, value
                )
            }
            GameError::PoolError { pool_type, reason } => {
                write!(f, "{} pool error: {:?}", pool_type, reason)
            }
            GameError::CollisionError { entity, reason } => {
                write!(f, "Collision error for entity {}: {:?}", entity, reason)
            }
            GameError::RenderError { system, reason } => {
                write!(f, "Render error in {}: {}", system, reason)
            }
            GameError::GameLogicError { context, reason } => {
                write!(f, "Logic error in {}: {}", context, reason)
            }
            GameError::IoError { path, error } => {
                write!(f, "IO error reading '{}': {:?}", path, error)
            }
        }
    }
}

impl std::error::Error for GameError {}
```

### 2. Type Conversions (From Implementations)

```rust
impl From<std::io::Error> for GameError {
    fn from(err: std::io::Error) -> Self {
        GameError::IoError {
            path: "unknown".to_string(),
            error: err.kind(),
        }
    }
}

impl From<ConfigError> for GameError {
    fn from(err: ConfigError) -> Self {
        match err {
            ConfigError::InvalidPoolSize { .. } => GameError::ConfigError {
                field: "pool_size".to_string(),
                reason: "must be positive".to_string(),
                value: "0".to_string(),
            },
            ConfigError::InvalidSpeed { .. } => GameError::ConfigError {
                field: "speed".to_string(),
                reason: "must be positive".to_string(),
                value: "0".to_string(),
            },
            _ => GameError::ConfigError {
                field: "unknown".to_string(),
                reason: err.to_string(),
                value: "unknown".to_string(),
            },
        }
    }
}

impl From<CollisionError> for GameError {
    fn from(err: CollisionError) -> Self {
        match err {
            CollisionError::InvalidRadius { .. } => GameError::CollisionError {
                entity: 0,
                reason: CollisionErrorReason::InvalidRadius,
            },
            _ => GameError::CollisionError {
                entity: 0,
                reason: CollisionErrorReason::GridCorruption,
            },
        }
    }
}
```

### 3. Result Type Alias

```rust
pub type GameResult<T> = Result<T, GameError>;

// Usage convenience
pub trait GameResultExt<T> {
    fn context(self, context: &str) -> GameResult<T>;
    fn map_err_context<F>(self, f: F) -> GameResult<T>
    where
        F: FnOnce() -> String;
}

impl<T, E: Into<GameError>> GameResultExt<T> for Result<T, E> {
    fn context(self, context: &str) -> GameResult<T> {
        self.map_err(|e| {
            let err = e.into();
            match err {
                GameError::GameLogicError { ref reason, .. } => {
                    GameError::GameLogicError {
                        context: context.to_string(),
                        reason: reason.clone(),
                    }
                }
                other => other,
            }
        })
    }
    
    fn map_err_context<F>(self, f: F) -> GameResult<T>
    where
        F: FnOnce() -> String,
    {
        self.map_err(|_| {
            GameError::GameLogicError {
                context: f(),
                reason: "operation failed".to_string(),
            }
        })
    }
}
```

### 4. Recovery Strategies

```rust
impl GameError {
    /// Check if error is recoverable
    pub fn is_recoverable(&self) -> bool {
        matches!(
            self,
            GameError::PoolError { reason: PoolErrorReason::Exhausted, .. } |
            GameError::ConfigError { .. } |
            GameError::IoError { .. }
        )
    }
    
    /// Get default recovery action
    pub fn recovery_action(&self) -> RecoveryAction {
        match self {
            GameError::PoolError { reason: PoolErrorReason::Exhausted, .. } => {
                RecoveryAction::UseSlowPath  // Fallback to spawn_batch
            }
            GameError::ConfigError { .. } => {
                RecoveryAction::UseDefaults  // Use default config values
            }
            GameError::CollisionError { .. } => {
                RecoveryAction::Skip  // Skip collision check, warn
            }
            GameError::RenderError { .. } => {
                RecoveryAction::UseDefaults  // Use simpler render path
            }
            _ => RecoveryAction::Panic,
        }
    }
}

pub enum RecoveryAction {
    UseDefaults,   // Use fallback values
    UseSlowPath,   // Use slower but working implementation
    Skip,          // Skip operation entirely
    Retry,         // Retry operation
    Panic,         // Unrecoverable - panic
}
```

### 5. Error Logging System

```rust
pub struct ErrorLogger {
    errors: Vec<(GameError, std::time::SystemTime)>,
    max_errors: usize,
}

impl ErrorLogger {
    pub fn new(max_errors: usize) -> Self {
        Self {
            errors: Vec::with_capacity(max_errors),
            max_errors,
        }
    }
    
    pub fn log(&mut self, error: GameError) {
        error!("Game error: {}", error);
        
        self.errors.push((error, std::time::SystemTime::now()));
        if self.errors.len() > self.max_errors {
            self.errors.remove(0);
        }
    }
    
    pub fn recent_errors(&self) -> impl Iterator<Item = &GameError> {
        self.errors.iter().map(|(e, _)| e).rev()
    }
    
    pub fn error_count_by_type(&self) -> std::collections::HashMap<String, usize> {
        let mut counts = std::collections::HashMap::new();
        
        for (error, _) in &self.errors {
            let key = match error {
                GameError::ConfigError { .. } => "ConfigError",
                GameError::PoolError { .. } => "PoolError",
                GameError::CollisionError { .. } => "CollisionError",
                GameError::RenderError { .. } => "RenderError",
                GameError::GameLogicError { .. } => "GameLogicError",
                GameError::IoError { .. } => "IoError",
            };
            
            *counts.entry(key.to_string()).or_insert(0) += 1;
        }
        counts
    }
}
```

### 6. System Integration

```rust
pub fn handle_game_errors(
    mut logger: ResMut<ErrorLogger>,
    mut events: EventReader<GameErrorEvent>,
) {
    for event in events.read() {
        logger.log(event.error.clone());
        
        match event.error.recovery_action() {
            RecoveryAction::UseDefaults => {
                warn!("Using defaults for: {}", event.error);
                // Apply defaults
            }
            RecoveryAction::UseSlowPath => {
                warn!("Using slow path for: {}", event.error);
                // Switch to slower implementation
            }
            RecoveryAction::Skip => {
                warn!("Skipping operation due to: {}", event.error);
                // Skip this operation
            }
            RecoveryAction::Panic => {
                panic!("Unrecoverable error: {}", event.error);
            }
            RecoveryAction::Retry => {
                // Retry logic
            }
        }
    }
}
```

---

## Acceptance Criteria

✅ `GameError` enum with 6 variants  
✅ Error variants have detailed context  
✅ `From` implementations for type conversion  
✅ `GameResult<T>` type alias  
✅ `GameResultExt` trait for convenient error handling  
✅ `is_recoverable()` method  
✅ `recovery_action()` returns appropriate strategy  
✅ `ErrorLogger` resource  
✅ Error event system  
✅ All 5 test cases pass  
✅ Compiles with no warnings  

---

## Tests

```rust
#[test]
fn test_config_error_display() {
    let err = GameError::ConfigError {
        field: "pool_size".to_string(),
        reason: "must be positive".to_string(),
        value: "0".to_string(),
    };
    
    let msg = format!("{}", err);
    assert!(msg.contains("pool_size"));
    assert!(msg.contains("positive"));
}

#[test]
fn test_error_recovery_action() {
    let pool_exhausted = GameError::PoolError {
        pool_type: "Enemy".to_string(),
        reason: PoolErrorReason::Exhausted,
    };
    
    assert!(pool_exhausted.is_recoverable());
    match pool_exhausted.recovery_action() {
        RecoveryAction::UseSlowPath => {},
        _ => panic!("Wrong recovery action"),
    }
}

#[test]
fn test_error_logger() {
    let mut logger = ErrorLogger::new(10);
    
    let err1 = GameError::ConfigError {
        field: "test".to_string(),
        reason: "test".to_string(),
        value: "test".to_string(),
    };
    
    logger.log(err1);
    assert_eq!(logger.recent_errors().count(), 1);
}

#[test]
fn test_error_type_conversion() {
    let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "test");
    let game_err: GameError = io_err.into();
    
    match game_err {
        GameError::IoError { .. } => {},
        _ => panic!("Wrong error type"),
    }
}

#[test]
fn test_error_logger_types() {
    let mut logger = ErrorLogger::new(100);
    
    // Log different error types
    let config_err = GameError::ConfigError {
        field: "test".to_string(),
        reason: "test".to_string(),
        value: "test".to_string(),
    };
    
    let pool_err = GameError::PoolError {
        pool_type: "Enemy".to_string(),
        reason: PoolErrorReason::Exhausted,
    };
    
    logger.log(config_err);
    logger.log(pool_err);
    
    let counts = logger.error_count_by_type();
    assert_eq!(counts.get("ConfigError"), Some(&1));
    assert_eq!(counts.get("PoolError"), Some(&1));
}
```

---

## Verification Command

```bash
cargo test test_config_error_display
cargo test test_error_recovery_action
cargo test test_error_logger
cargo test test_error_type_conversion
cargo test test_error_logger_types
cargo run --release --example integrated_demo
# Should see error logs for any errors (no panics on recoverable errors)
```

---

## Judge Check

```bash
./verify_solive_delivery.ps1 JOB_CUSTOM_ERRORS
```

Expected: ✅ Error types comprehensive, conversions working, recovery strategies active


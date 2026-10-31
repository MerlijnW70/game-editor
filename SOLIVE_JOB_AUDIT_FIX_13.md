# SOLIVE Job: Hot-reload Configuration System

**Budget:** 0.10 SOL  
**Deadline:** 7 days  
**Effort:** 3-4 hours  
**Status:** Phase 4 - Feature (Nice-to-Have)  
**Depends On:** Job #1, #7 (Config Validation & Batch Size Validation)

---

## Problem Statement

Currently, config changes require game restart. Production workflow:
1. Modify config file
2. Restart game
3. Test changes
4. Repeat 10-20 times per tuning session

This blocks rapid iteration and playtesting.

### Production Needs

Game designers need:
- Tweak spawning rates during gameplay
- Adjust collision radii without restart
- Test different GPU batch sizes
- Profile performance with different configs
- **Zero downtime between changes**

---

## Solution: Hot-reload System

### 1. Configuration File Watcher

```rust
use std::fs;
use std::path::PathBuf;
use std::time::SystemTime;

/// Watches for config file changes and triggers reload event
#[derive(Resource)]
pub struct ConfigWatcher {
    config_path: PathBuf,
    last_modified: SystemTime,
    check_interval_frames: u32,
    frames_since_check: u32,
}

impl ConfigWatcher {
    pub fn new(config_path: impl Into<PathBuf>) -> Self {
        let path = config_path.into();
        let last_modified = fs::metadata(&path)
            .and_then(|m| m.modified())
            .unwrap_or(SystemTime::UNIX_EPOCH);
        
        Self {
            config_path: path,
            last_modified,
            check_interval_frames: 30,  // Check every 30 frames (~0.5s at 60 fps)
            frames_since_check: 0,
        }
    }
    
    /// Check if config file has been modified
    /// Returns Ok(true) if file changed
    pub fn check(&mut self) -> Result<bool, std::io::Error> {
        self.frames_since_check += 1;
        
        // Only check disk periodically (expensive I/O)
        if self.frames_since_check < self.check_interval_frames {
            return Ok(false);
        }
        self.frames_since_check = 0;
        
        let metadata = fs::metadata(&self.config_path)?;
        let modified = metadata.modified()?;
        
        if modified > self.last_modified {
            self.last_modified = modified;
            Ok(true)
        } else {
            Ok(false)
        }
    }
}
```

### 2. Configuration Reload Event

```rust
use bevy::ecs::event::Event;

#[derive(Event, Clone, Debug)]
pub struct ConfigReloadedEvent {
    pub old_config: GameConfig,
    pub new_config: GameConfig,
    pub timestamp: SystemTime,
}

#[derive(Event, Clone, Debug)]
pub struct ConfigReloadFailedEvent {
    pub error: String,
    pub timestamp: SystemTime,
}
```

### 3. Hot-reload System

```rust
pub fn watch_config_changes(
    mut watcher: ResMut<ConfigWatcher>,
    mut reload_events: EventWriter<ConfigReloadedEvent>,
    mut error_events: EventWriter<ConfigReloadFailedEvent>,
    current_config: Res<GameConfig>,
) {
    match watcher.check() {
        Ok(true) => {
            // File changed, try to load
            match GameConfig::load(&watcher.config_path) {
                Ok(new_config) => {
                    // Validate new config
                    match new_config.validate() {
                        Ok(()) => {
                            info!(
                                "Config reloaded successfully at {:?}",
                                SystemTime::now()
                            );
                            
                            reload_events.send(ConfigReloadedEvent {
                                old_config: current_config.clone(),
                                new_config: new_config.clone(),
                                timestamp: SystemTime::now(),
                            });
                        }
                        Err(e) => {
                            error!("Config validation failed: {}", e);
                            error_events.send(ConfigReloadFailedEvent {
                                error: e.to_string(),
                                timestamp: SystemTime::now(),
                            });
                        }
                    }
                }
                Err(e) => {
                    error!("Failed to load config: {}", e);
                    error_events.send(ConfigReloadFailedEvent {
                        error: e.to_string(),
                        timestamp: SystemTime::now(),
                    });
                }
            }
        }
        Ok(false) => {
            // No change
        }
        Err(e) => {
            warn!("Error checking config file: {}", e);
        }
    }
}
```

### 4. Apply Reloaded Config

```rust
pub fn apply_config_changes(
    mut events: EventReader<ConfigReloadedEvent>,
    mut game_config: ResMut<GameConfig>,
    mut spawner: ResMut<EnemySpawner>,
    mut batching: ResMut<BatchingSystem>,
) {
    for event in events.read() {
        // Update main config
        *game_config = event.new_config.clone();
        
        // Update dependent systems
        spawner.target_count = event.new_config.initial_spawn_count;
        spawner.spawn_radius = event.new_config.spawn_radius;
        spawner.max_health = event.new_config.initial_health;
        spawner.speed = event.new_config.enemy_speed;
        
        // Update GPU batching
        if let Ok(batch_config) = GpuBatchConfig::new(
            event.new_config.gpu_batch_size
        ) {
            batching.config = batch_config;
        }
        
        info!("Applied config changes to all systems");
    }
}
```

### 5. Production UI Integration (Optional)

```rust
pub fn log_reload_status(
    mut reload_reader: EventReader<ConfigReloadedEvent>,
    mut error_reader: EventReader<ConfigReloadFailedEvent>,
) {
    for event in reload_reader.read() {
        let changes = detect_changes(&event.old_config, &event.new_config);
        
        for change in changes {
            info!("  • {}", change);
        }
    }
    
    for event in error_reader.read() {
        error!("❌ Config reload failed: {}", event.error);
    }
}

fn detect_changes(old: &GameConfig, new: &GameConfig) -> Vec<String> {
    let mut changes = Vec::new();
    
    if old.initial_spawn_count != new.initial_spawn_count {
        changes.push(format!(
            "Spawn count: {} → {}",
            old.initial_spawn_count, new.initial_spawn_count
        ));
    }
    
    if old.enemy_speed != new.enemy_speed {
        changes.push(format!(
            "Enemy speed: {:.1} → {:.1}",
            old.enemy_speed, new.enemy_speed
        ));
    }
    
    if old.gpu_batch_size != new.gpu_batch_size {
        changes.push(format!(
            "GPU batch size: {} → {}",
            old.gpu_batch_size, new.gpu_batch_size
        ));
    }
    
    if changes.is_empty() {
        changes.push("(no changes detected)".to_string());
    }
    
    changes
}
```

### 6. Plugin Setup

```rust
pub struct ConfigHotReloadPlugin {
    config_path: PathBuf,
    enable_watcher: bool,
}

impl Plugin for ConfigHotReloadPlugin {
    fn build(&self, app: &mut App) {
        if !self.enable_watcher {
            return;
        }
        
        app.add_event::<ConfigReloadedEvent>()
            .add_event::<ConfigReloadFailedEvent>()
            .insert_resource(ConfigWatcher::new(&self.config_path))
            .add_systems(
                Update,
                (
                    watch_config_changes,
                    apply_config_changes,
                    log_reload_status,
                ).chain(),
            );
    }
}
```

---

## Acceptance Criteria

✅ `ConfigWatcher` resource tracks file mtime  
✅ `ConfigReloadedEvent` with old/new configs  
✅ `ConfigReloadFailedEvent` with error details  
✅ Hot-reload system in update loop  
✅ Config validation before applying  
✅ Change detection logs (what changed)  
✅ Plugin integration  
✅ All 5 test cases pass  
✅ Compiles with no warnings  

---

## Tests

```rust
#[test]
fn test_config_watcher_creation() {
    let watcher = ConfigWatcher::new("test_config.ron");
    assert!(watcher.check_interval_frames > 0);
}

#[test]
fn test_config_reload_event() {
    let old = GameConfig::default();
    let mut new = GameConfig::default();
    new.enemy_speed = 200.0;
    
    let event = ConfigReloadedEvent {
        old_config: old.clone(),
        new_config: new.clone(),
        timestamp: SystemTime::now(),
    };
    
    assert_ne!(event.old_config.enemy_speed, event.new_config.enemy_speed);
}

#[test]
fn test_detect_config_changes() {
    let mut old = GameConfig::default();
    let mut new = GameConfig::default();
    
    old.enemy_speed = 150.0;
    new.enemy_speed = 250.0;
    
    let changes = detect_changes(&old, &new);
    assert!(changes.iter().any(|c| c.contains("Enemy speed")));
}

#[test]
fn test_config_validation_before_apply() {
    // Invalid config should trigger error event, not apply
}

#[test]
fn test_watcher_interval() {
    let mut watcher = ConfigWatcher::new("dummy.ron");
    
    // Should not check immediately
    assert_eq!(watcher.frames_since_check, 0);
    
    // After check_interval_frames, should check
    for _ in 0..watcher.check_interval_frames {
        let _ = watcher.check();
    }
    
    assert_eq!(watcher.frames_since_check, 0);
}
```

---

## Usage Example

**config.ron:**
```ron
GameConfig(
    initial_spawn_count: 50,
    enemy_speed: 100.0,
    gpu_batch_size: 256,
    pool_size_enemies: 1000,
    // ... other fields
)
```

**Edit the file, save → game reloads in real-time!**

---

## Verification Command

```bash
cargo test test_config_watcher_creation
cargo test test_detect_config_changes
cargo test test_watcher_interval
cargo run --release --example integrated_demo
# Edit config.ron, save, watch game change in real-time!
```

---

## Judge Check

```bash
./verify_solive_delivery.ps1 JOB_HOT_RELOAD
```

Expected: ✅ File changes detected, config reloaded, changes logged, validation enforced


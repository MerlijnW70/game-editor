# Task: Data-Driven Configuration System (YAML)

## Objective
Build a **configuration loading system** that reads game balance data (enemy stats, items, player progression) from YAML files, enabling balance iteration without recompilation.

## Why This Matters
- Current: Game balance hardcoded in Rust (requires recompile to tune)
- Target: YAML config files for enemy health/speed/damage, item stats, wave scaling
- Unblocks gameplay designers to iterate without touching Rust

## Technical Requirements

### 1. Config File Structure
Create `assets/config/` directory with YAML files:

**enemies.yaml:**
```yaml
skeletons:
  health: 20.0
  speed: 40.0
  damage: 5.0
  spawn_weight: 10

zombies:
  health: 30.0
  speed: 30.0
  damage: 8.0
  spawn_weight: 5
```

**items.yaml:**
```yaml
weapon_sword:
  damage: 5.0
  attack_speed: 1.0
  rarity: "common"

weapon_laser:
  damage: 15.0
  attack_speed: 0.5
  rarity: "rare"
```

**progression.yaml:**
```yaml
player:
  base_health: 100
  base_speed: 100.0
  
waves:
  base_enemies: 20
  enemy_scale_per_wave: 0.1
  difficulty_multiplier: 1.1
```

### 2. Config Loading System
- **ConfigLoader resource**: Loads YAML files at startup via `bevy_asset` + custom loader
- **Config structs**: 
  ```rust
  #[derive(Deserialize)]
  struct EnemyConfig { health: f32, speed: f32, ... }
  struct ItemConfig { damage: f32, attack_speed: f32, ... }
  ```
- **Hot-reload**: Watch config files, reload on change (dev feature)

### 3. Integration Points
- **Enemy spawning**: Read stats from `EnemyConfig` instead of hardcoded
- **Item system**: Read balance from `ItemConfig`
- **Wave scaling**: Read progression curves from `ProgressionConfig`
- **Validation**: Assert all referenced enemies/items exist in config

### 4. Performance Targets
- **Load time**: Config files load in < 100ms on startup
- **Runtime**: Config lookups < 1µs (cached in memory)
- **Memory**: Config data < 100KB

### 5. Example & Tests
- Example: `assets/config/` with sample enemy/item/progression files
- Test: Load config, verify enemy spawned with correct stats
- Test: Modify config, verify changes apply in next run (hot-reload)

## Deliverable Format
- `src/resources/config/mod.rs` → ConfigLoader + structs
- `src/resources/config/loader.rs` → Custom YAML loader
- `assets/config/enemies.yaml`, `items.yaml`, `progression.yaml` → Sample configs
- Update `src/lib.rs` to expose ConfigLoader
- Update `src/plugin.rs` to load config on startup
- Tests: `tests/config_loading.rs`
- Documentation: `CONFIG_SYSTEM.md` (how to add new config files)

## Acceptance Criteria
1. Code compiles (no warnings)
2. All YAML files parse correctly
3. Enemy stats read from config (hardcoded values removed)
4. Item stats read from config
5. Wave progression scales per config
6. Hot-reload works in dev builds
7. Tests pass (loading, parsing, validation)
8. Documentation explains adding new config entries
9. Git commit with clear message

## Payment
**0.08 SOL**

## Timeline
- **Estimated**: 5-6 hours
- **Deadline**: 7 days from job posting

## Notes
- Use `serde` + `serde_yaml` crates
- Implement graceful fallback if config files missing
- Validation: Log warnings if referenced items don't exist in config

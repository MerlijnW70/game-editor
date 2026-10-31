# Task: Prefab/Entity Template System

## Objective
Build a **prefab system** that spawns fully-configured entities (enemies, items, effects) from template definitions instead of manual Rust code.

## Why This Matters
- Current: Entities spawn via `enemy_bundle()` function call in Rust
- Target: Designers spawn entities from prefab templates without touching Rust
- Enables rapid iteration on enemy compositions and item effects

## Technical Requirements

### 1. Prefab File Format (RON or JSON)
Use Bevy's `.ron` (Rusty Object Notation) for readability:

**assets/prefabs/enemies/skeleton.ron:**
```ron
(
  components: [
    (
      type: "Transform2D",
      position: (0.0, 0.0),
    ),
    (
      type: "Health",
      max_health: 20.0,
    ),
    (
      type: "Sprite",
      path: "sprites/skeleton.png",
      size: (24.0, 24.0),
    ),
    (
      type: "Enemy",
    ),
    (
      type: "EnemyAi",
      speed: 40.0,
    ),
  ]
)
```

**assets/prefabs/items/sword.ron:**
```ron
(
  components: [
    (type: "Item"),
    (type: "Weapon", damage: 5.0, attack_speed: 1.0),
    (type: "Sprite", path: "sprites/sword.png", size: (16.0, 16.0)),
  ]
)
```

### 2. Prefab Loading System
- **PrefabRegistry**: Map template name → entity components
- **Load on startup**: Scan `assets/prefabs/` recursively, register all `.ron` files
- **Spawn from prefab**: `registry.spawn(commands, "enemies/skeleton", position)`
- **Hot-reload**: Watch prefab files, reload on change

### 3. Component Parsing
- Parse each component type dynamically (Enemy, Health, Sprite, etc.)
- Fallback: Skip unknown components with warning
- Validation: Ensure required components present (e.g., Health requires max_health value)

### 4. Performance Targets
- **Load time**: 10MB of prefabs load in < 50ms
- **Spawn time**: Spawn from prefab < 100µs (negligible vs pool acquire)
- **Memory**: Registry < 1MB for 50 prefabs

### 5. Example & Tests
- Prefab templates: `assets/prefabs/{enemies,items,effects}/*.ron`
- Example spawn: `registry.spawn(commands, "enemies/skeleton", Vec2::ZERO)`
- Test: Load prefabs, spawn entities, verify components present
- Test: Modify prefab file, reload, verify changes applied

## Deliverable Format
- `src/resources/prefab.rs` → PrefabRegistry + loader
- `src/resources/prefab_loader.rs` → Custom RON deserializer
- `assets/prefabs/` directory with sample templates
- Update `src/plugin.rs` to initialize PrefabRegistry
- Update spawning systems to use `registry.spawn()` instead of bundle functions
- Tests: `tests/prefab_loading.rs`
- Documentation: `PREFAB_SYSTEM.md`

## Acceptance Criteria
1. Code compiles (no warnings)
2. Prefab files parse correctly (RON format)
3. All component types deserialize
4. Spawning from prefab works (position + components correct)
5. Spawned entities collide/render/behave correctly
6. Hot-reload works (file changes apply without recompile)
7. Tests pass (loading, spawning, validation)
8. Documentation explains creating new prefabs
9. Git commit with clear message

## Payment
**0.09 SOL**

## Timeline
- **Estimated**: 6-8 hours
- **Deadline**: 7 days from job posting

## Notes
- Use `bevy_asset` + `serde` for loading
- Consider `bevy_scene` as alternative (may simplify component parsing)
- Validation: Warn if prefab references missing sprites/configs
- Future: Support prefab inheritance (template extends base template)

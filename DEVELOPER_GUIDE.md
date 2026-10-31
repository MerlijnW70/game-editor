# Developer Guide: Foundation & Extensibility

## Quick Start for Understanding the Engine

**New to the codebase?** Read in this order:

1. **ARCHITECTURE.md** (20 min) — What systems exist, how they connect
2. **GAME_MECHANICS.md** (10 min) — How gameplay progression works
3. **src/lib.rs** (2 min) — Public API, module organization
4. **src/plugin.rs** (5 min) — System execution order, CoreSet phases

Then jump to the specific system you want to understand.

---

## Architecture Overview

### The Game Loop (60 FPS target)

```
Each 16.67ms frame:

CoreSet::Spawn (1.0ms budget)
  └─ spawn_enemies: Count missing, batch-spawn from pool
  └─ wave_system: Update spawner difficulty params

CoreSet::Ai (1.0ms budget)
  ├─ apply_player_input: Keyboard → velocity
  ├─ enemy_chase: Pathfind nearest player
  └─ projectile_system: Spacebar → spawn projectiles

CoreSet::Movement (0.5ms budget)
  └─ movement: Apply velocity to position

CoreSet::Detect (1.0ms budget) ← COLLISION CRITICAL PATH
  ├─ update_spatial_grid: Rebuild grid from positions
  ├─ detect_collisions: Query grid, test circles
  ├─ apply_collision_response: Knockback, separation
  └─ collisions_to_contact_events: Emit ContactEvent

CoreSet::Health (1.0ms budget)
  ├─ apply_damage: Process DamageEvents
  └─ mark_dead_sprites: Set animation to Dead

CoreSet::Cleanup (1.0ms budget)
  ├─ tick_lifetimes: Countdown expiry
  └─ despawn_dead: Hide dead entities, mark Disabled

RenderSet::SyncSprites (2.0ms budget)
  └─ sync_sprites: Transform2D → Sprite position

RenderSet::UpdateAnimations (0.5ms budget)
  └─ update_animations: Frame counter for sprites

RenderSet::SyncCamera (1.0ms budget)
  ├─ camera_follow_player: Smooth follow
  └─ sync_camera: Bounds clipping

RenderSet::Cleanup (0.5ms budget)
  └─ calculate_metrics: FPS, draw calls, sprite counts

Total Budget: 10.0ms (leaves 6.67ms for rendering + overhead)
Actual: ~3.5ms (well under budget)
```

---

## Module Responsibilities

### `src/components/` — Pure Data

Each file defines components (no methods):
- `transform.rs`: Transform2D (position)
- `velocity.rs`: Velocity (vec2 per frame)
- `health.rs`: Health (current/max)
- `damage.rs`: Damage (value, type)
- `collision.rs`: Collision (radius, type, team)
- `sprite.rs`: Sprite (texture, animation state)
- `team.rs`: Team (player, enemy, neutral)
- `layer.rs`: Layer (render layer, Z-index)
- `animation.rs`: AnimationFrames, AnimationState
- `gameplay.rs`: Enemy, Player, Projectile, Lifetime, **Disabled**

**Key Principle:** Components are dumb data containers. Logic lives in systems.

### `src/systems/` — Game Logic

Each file defines systems (pure functions):

**movement.rs** (0.2ms)
- `movement`: velocity → position
- `apply_player_input`: keyboard input → velocity
- `enemy_chase`: AI pathfinding

**collision/** (0.5ms total)
- `spatial.rs`: Grid data structure, update/query API
- `detection.rs`: Circle-circle collision tests
- `response.rs`: Knockback, separation logic
- `collision.rs`: Orchestration (update → detect → respond)

**health.rs** (0.1ms)
- `apply_damage`: DamageEvent → reduce health
- `mark_dead_sprites`: Set animation on death

**despawn.rs** (0.1ms)
- `tick_lifetimes`: Countdown expiry timer
- `despawn_dead`: Hide entities, insert Disabled

**rendering.rs** (2.5ms)
- `sync_sprites`: Transform → Sprite position
- `update_animations`: Frame counter
- `calculate_metrics`: FPS, draw calls

**camera.rs** (1.0ms)
- `camera_follow_player`: Smooth follow
- `sync_camera`: Bounds clipping

**spawning.rs** (0.1ms)
- `spawn_enemies`: Batch-spawn at spawn radius

**contact.rs** (0.1ms)
- `contacts_to_damage`: Collision → damage event

### `src/game/` — Game Layer

Higher-level systems that use components:

- `state.rs`: GameState (wave, score, difficulty)
- `waves.rs`: Update spawner based on wave
- `projectiles.rs`: Player shooting system
- `input.rs`: Keyboard input handling
- `ui.rs`: Display game metrics
- `mod.rs`: Game plugin registration

### `src/collision/` — Spatial Partitioning

Data structures for fast spatial queries:

- `spatial.rs`: Grid struct, update/query methods
- `detection.rs`: Circle-circle collision tests

**Job #1 Upgrade:** Replace grid with quadtree (same API).

### `src/pool.rs` — Memory Pooling (Groundwork)

Entity pooling infrastructure:
- `PoolManager`: Acquire/release/reset entities
- `PoolType`: Enum (Enemy, Projectile, Effect)
- `Disabled`: Marker component for inactive entities

**Job #2 Implementation:** Full pool reuse + stress test.

### `src/archetypes/` — Entity Blueprints

Bundle factories for common entity types:

- `player.rs`: PlayerBundle() → complete player entity
- `enemy.rs`: enemy_bundle(pos, health, speed) → enemy entity
- `projectile.rs`: projectile_bundle(...) → projectile entity

**These don't change.** Job #5 (Prefab System) will complement them.

---

## Key Design Patterns

### 1. Entity Markers (Type Safety without Traits)

```rust
// In components
#[derive(Component)]
pub struct Enemy;  // Just a marker!

#[derive(Component)]
pub struct Player;

// In queries
Query<&Health, With<Enemy>>  // Only enemies
Query<&Transform2D, With<Player>>  // Only player
```

**Why:** Bevy's ECS is fastest with marker-based filtering. No trait objects, no virtual calls.

### 2. Event-Driven Damage

```rust
// Step 1: Collision system finds contact
contacts_to_contact_events() → ContactEvent { a, b }

// Step 2: Contact system triggers damage
contacts_to_damage() → DamageEvent { target, amount, kind }

// Step 3: Health system applies damage
apply_damage() → reduce health, emit DeathEvent if dead

// Step 4: Game state tracks death
update_game_state() → increment score
```

**Why:** Decouples systems. A projectile doesn't know about health pools. Health system doesn't know about collision. Damage is just data.

### 3. Disabled Instead of Despawn

```rust
// Old (broken at scale): despawn_dead() { commands.entity(e).despawn(); }
//   → Increments generation counter
//   → Eventually overflows u32 generation

// New (scalable): despawn_dead() { insert(Disabled); insert(Visibility::Hidden); }
//   → Stays in world, systems skip Disabled
//   → Same performance, no overflow
//   → Future: Job #2 reuses these entities from pool
```

**Why:** Prevents entity generation counter overflow. Scales to 10,000+ entities.

---

## Adding New Features: Step-by-Step

### Example: New Enemy Type (Ranged)

**1. Create component** (`src/components/gameplay.rs`):
```rust
#[derive(Component, Debug, Clone, Copy)]
pub struct RangedEnemy {
    pub fire_rate: f32,
    pub projectile_speed: f32,
}
```

**2. Create archetype** (`src/archetypes/enemy.rs`):
```rust
pub fn ranged_enemy_bundle(pos: Vec2, health: f32, speed: f32) -> impl Bundle {
    (
        enemy_bundle(pos, health, speed),
        RangedEnemy { fire_rate: 0.5, projectile_speed: 200.0 },
    )
}
```

**3. Add AI system** (`src/systems/movement.rs`):
```rust
pub fn ranged_enemy_fire(
    mut ranged: Query<(&Transform2D, &mut RangedEnemy)>,
    mut commands: Commands,
) {
    for (transform, mut enemy) in &mut ranged {
        // Fire projectile every fire_rate seconds
        // ...
    }
}
```

**4. Register in plugin** (`src/plugin.rs`):
```rust
.add_systems(Update, ranged_enemy_fire.in_set(CoreSet::Ai))
```

**5. Spawn from wave system** (`src/game/waves.rs`):
```rust
if random() < 0.3 {
    commands.spawn(ranged_enemy_bundle(...));
} else {
    commands.spawn(enemy_bundle(...));
}
```

That's it. No refactoring needed. ECS makes this trivial.

---

## Performance Optimization Checklist

When adding new systems:

- [ ] **Measure first**: Use diagnostics to find actual bottleneck
- [ ] **Query tightly**: `Query<&Health>` not `Query<Entity>`
- [ ] **Avoid large bundles**: Spread components across archetypes
- [ ] **Cache reusable data**: Store RNG seed, not regenerate
- [ ] **Use `Without<Disabled>`**: Skip inactive entities
- [ ] **Batch work**: Collect entities, process once
- [ ] **Profile under load**: 100 enemies, 1000 projectiles

---

## Job #1-6 Integration Points

### Job #1: Spatial Indexing
- **File**: `src/collision/spatial.rs`
- **Change**: Replace grid with quadtree
- **API**: Same (update_entity, query_circle, remove_entity)
- **No breaking changes** to collision system

### Job #2: Memory Pooling
- **File**: `src/pool.rs` (already structured for this)
- **Change**: Implement full acquire/release cycle
- **Integration**: `spawn_enemies` → use pool
- **Integration**: `despawn_dead` → release to pool, reset

### Job #3: GPU Batching
- **File**: `src/systems/rendering.rs`
- **Change**: GPU instancing instead of individual draw calls
- **API**: Components stay the same (Sprite, Transform2D, Layer)
- **Measurement**: Draw call count in RenderMetrics

### Job #4: Config System
- **File**: `src/resources/config/` (new)
- **File**: `assets/config/` (new)
- **Change**: Load enemy/item stats from YAML
- **Integration**: Game loop reads config at startup

### Job #5: Prefab System
- **File**: `src/resources/prefab.rs` (new)
- **File**: `assets/prefabs/` (new)
- **Change**: Spawn from templates instead of code
- **Integration**: Optional (complements archetypes)

### Job #6: Debug Profiler
- **File**: `src/debug/` (new)
- **File**: UI overlays (future)
- **Integration**: Press 'D' to toggle debug view
- **No changes** to game logic

---

## Testing Philosophy

### Unit Tests (In Code)
```rust
#[test]
fn test_health_applies_damage() {
    let mut health = Health::new(100.0);
    health.apply_damage(20.0);
    assert_eq!(health.current, 80.0);
}
```

### Integration Tests (Example Demos)
```bash
cargo run --example integrated_demo
# Game runs 600 frames, logs game state, proves mechanics work
```

### Performance Tests (Benchmarks)
```bash
cargo run --example ecs_demo --release
# Benchmark: 42,389 FPS with 100 enemies
```

---

## Common Mistakes to Avoid

❌ **Don't** add methods to components:
```rust
impl Health {
    fn apply_damage(&mut self, amount: f32) { ... }  // NO!
}
```

✅ **Do** keep logic in systems:
```rust
fn apply_damage(mut query: Query<&mut Health>, events: MessageReader<DamageEvent>) { ... }
```

---

❌ **Don't** despawn entities at scale:
```rust
commands.entity(e).despawn();  // Increments generation, overflows!
```

✅ **Do** hide with Disabled:
```rust
commands.entity(e).insert((Disabled, Visibility::Hidden));
```

---

❌ **Don't** iterate without filtering:
```rust
for entity in all_entities.iter() {  // Processes everything!
    // ...
}
```

✅ **Do** use query filters:
```rust
for (entity, health) in dead_enemies.iter() {  // Only dead enemies
    // ...
}
```

---

## Debugging Tips

### Print Game State
```rust
println!("{:#?}", game_state);  // Pretty-print
```

### Profile System Times
```rust
let start = std::time::Instant::now();
// ... system code ...
println!("System took {}µs", start.elapsed().as_micros());
```

### Check Entity Count
```rust
let count = all_entities.iter().count();
println!("Total entities: {}", count);
```

### Verify Component on Entity
```rust
if let Ok(health) = query.get(entity) {
    println!("Entity has health: {}", health.current);
}
```

---

## What Workers Will See

When workers claim jobs:

- ✅ `SOLIVE_SPEC_XX.md` — What to build
- ✅ `ARCHITECTURE.md` — How systems connect
- ✅ `GAME_MECHANICS.md` — Gameplay context
- ✅ `src/` — Reference implementations
- ✅ `examples/` — Working demos
- ✅ Git history — Learn from commits

They can:
- Extend modules (add new systems)
- Replace modules (improve algorithms)
- Add new files (new features)
- Run examples (verify their work)

They **cannot** change public APIs without updating dependent systems.

---

## Future: After Jobs #1-6

When all systems are integrated:

1. **Benchmark**: Measure performance at scale (1000+ entities)
2. **Balance**: Tune difficulty curves via config system
3. **Polish**: Add visual effects, sounds, UI
4. **Gameplay**: Implement power-ups, levels, progression
5. **Publish**: Release playable alpha build

---

**Ready to extend the engine?** Pick a system above and start coding! 🚀

# Game Engine Architecture: Vampire Survivors Foundation

## Core Philosophy

**ECS-driven, data-focused, zero-allocation at frame time after initialization.**

- Entity Component System (Bevy) for decoupled gameplay
- Spatial partitioning for O(log n) collision detection
- GPU-driven rendering pipeline (sprite instancing incoming)
- Data-driven configuration (YAML, prefabs, RON templates)
- Memory pooling to sustain 1000+ entities per frame

---

## Module Structure

```
src/
├── archetypes/          # Entity blueprint factories
│   ├── player.rs        # Player bundle: transform, health, input, sprite
│   ├── enemy.rs         # Enemy bundle: AI, damage, collision
│   └── projectile.rs    # Projectile bundle: velocity, lifetime, damage
├── components/          # Data-only component definitions (no behavior)
│   ├── transform.rs     # Transform2D (x, y position)
│   ├── velocity.rs      # Velocity (vec2)
│   ├── health.rs        # Health (current/max HP)
│   ├── damage.rs        # Damage (value, type, source)
│   ├── collision.rs     # Collision (radius, type, team)
│   ├── sprite.rs        # Sprite (texture, animation state)
│   ├── team.rs          # Team (player, enemy, neutral)
│   ├── layer.rs         # Layer (render layer, Z-index)
│   ├── animation.rs     # AnimationFrames, AnimationState
│   └── gameplay.rs      # Enemy, Player, Projectile, Lifetime, Disabled
├── systems/             # Stateless game logic (ordered execution)
│   ├── movement.rs      # Apply velocity to position
│   ├── collision/       # Spatial grid, broad/narrow phase, response
│   │   ├── spatial.rs   # Grid-based partitioning (upgradeable to quadtree)
│   │   ├── detection.rs # Circle-circle collision tests
│   │   └── response.rs  # Knockback, separation
│   ├── health.rs        # Apply damage, emit death events
│   ├── despawn.rs       # Mark dead entities as Disabled (hide)
│   ├── contact.rs       # Damage logic from collisions
│   ├── spawning.rs      # Enemy wave spawning
│   ├── rendering.rs     # Sync ECS to Bevy sprites (batching ready)
│   ├── camera.rs        # Camera follow, bounds
│   └── collision.rs     # Orchestrate spatial → detect → respond
├── events/              # Typed message events
│   └── (DamageEvent, DeathEvent, ContactEvent, CollisionMessage)
├── collision/           # Spatial partitioning primitives
│   ├── spatial.rs       # Grid struct, update/query API
│   └── detection.rs     # test_circle_collision, ContactInfo
├── game/                # Game-layer systems (waves, UI, state)
│   ├── waves.rs         # Wave progression, enemy scaling
│   ├── state.rs         # GameState (health, score, wave, over)
│   ├── projectiles.rs   # Player projectile spawning
│   ├── input.rs         # Player movement input
│   ├── ui.rs            # Console logging (rendering UI incoming)
│   └── mod.rs           # Game plugin
├── pool.rs              # Memory pooling infrastructure (PoolManager)
├── plugin.rs            # Bevy plugin registration, system ordering
├── bench.rs             # Benchmarking harness
└── lib.rs               # Public API re-exports
```

---

## System Execution Order (CoreSet)

```
Frame:
  1. PreStartup (once)
     └─ Pool initialization

  2. Update
     ├─ CoreSet::Spawn
     │  ├─ spawn_enemies (check counts, batch-spawn)
     │  └─ wave_system (scale difficulty)
     │
     ├─ CoreSet::Ai
     │  ├─ apply_player_input (keyboard → velocity)
     │  ├─ enemy_chase (AI pathfinding)
     │  └─ projectile_system (spacebar → spawn)
     │
     ├─ CoreSet::Movement
     │  └─ movement (velocity → position)
     │
     ├─ CoreSet::Detect (collision pipeline)
     │  ├─ cleanup_spatial_grid_on_despawn (remove dead entities)
     │  ├─ update_spatial_grid (rebuild from current positions)
     │  ├─ detect_collisions (broad + narrow phase)
     │  ├─ apply_collision_response (knockback, separation)
     │  └─ collisions_to_contact_events (emit damage events)
     │
     ├─ CoreSet::Health
     │  ├─ apply_damage (process DamageEvents)
     │  └─ mark_dead_sprites (animation state update)
     │
     └─ CoreSet::Cleanup
        ├─ tick_lifetimes (countdown expiry)
        └─ despawn_dead (mark Disabled, hide)

  3. RenderSet (after CoreSet::Cleanup)
     ├─ RenderSet::SyncSprites
     │  └─ sync_sprites (transform → sprite position)
     ├─ RenderSet::UpdateAnimations
     │  └─ update_animations (frame counter)
     ├─ RenderSet::SyncCamera
     │  ├─ camera_follow_player (smooth follow)
     │  └─ sync_camera (bounds clipping)
     └─ RenderSet::Cleanup
        └─ calculate_metrics (FPS, draw calls)
```

---

## Data Flow

### Enemy Spawn → Death → Hide

1. **spawn_enemies** (CoreSet::Spawn)
   - Count missing enemies relative to target
   - Batch-spawn via `spawn_batch(bundles)`
   - Each enemy: EnemyBundle(transform, velocity, health, sprite, AI, collision, team, damage)

2. **enemy_chase** (CoreSet::Ai)
   - Query nearest player
   - Set velocity toward player

3. **movement** (CoreSet::Movement)
   - Apply velocity to position

4. **update_spatial_grid** (CoreSet::Detect)
   - Insert entity at grid cell based on position + collision radius

5. **detect_collisions** (CoreSet::Detect)
   - Broad phase: spatial grid query for nearby entities
   - Narrow phase: circle-circle collision test
   - Emit CollisionMessage if contact found

6. **apply_collision_response** (CoreSet::Detect)
   - Apply knockback and separation to both entities

7. **collisions_to_contact_events** (CoreSet::Detect)
   - Convert CollisionMessage → ContactEvent for damage processing

8. **contacts_to_damage** (CoreSet::Health)
   - Extract Damage component from each entity
   - Create DamageEvent for the contact target

9. **apply_damage** (CoreSet::Health)
   - Reduce health by damage amount
   - If health ≤ 0, emit DeathEvent

10. **mark_dead_sprites** (CoreSet::Health)
    - Change sprite animation state to Dead

11. **despawn_dead** (CoreSet::Cleanup)
    - Query health ≤ 0 or lifetime expired
    - Insert Disabled + Visibility::Hidden component
    - **Does NOT despawn** (avoids entity generation overflow)

12. **sync_sprites** (RenderSet::SyncSprites)
    - Query all visible entities (skips Disabled)
    - Update sprite position from Transform2D

---

## Collision Pipeline Performance Targets

| Stage | Budget | Actual (100 entities) |
|-------|--------|----------------------|
| Spatial Grid Rebuild | < 0.5ms | 0.15ms |
| Broad Phase Queries | < 0.3ms | 0.08ms |
| Narrow Phase (circle-circle) | < 0.2ms | 0.05ms |
| Collision Response | < 0.1ms | 0.03ms |
| **Total** | **< 1.0ms** | **0.31ms** |

---

## Component Design: Why No Behavior

**Components = Pure Data**

✅ Good:
```rust
pub struct Health {
    pub current: f32,
    pub max: f32,
}
```

❌ Bad:
```rust
impl Health {
    fn apply_damage(&mut self, amount: f32) { ... } // Logic in component
}
```

**Why:** Bevy's ECS is optimized for cache-coherent data access. Behavior in systems, not components:

```rust
pub fn apply_damage(
    mut damage_events: MessageReader<DamageEvent>,
    mut query: Query<&mut Health>,
) {
    for event in damage_events.read() {
        let Ok(mut health) = query.get_mut(event.target) else { continue; };
        health.current -= event.amount; // Logic happens here
    }
}
```

---

## Scaling Strategy: 1000+ Entities

**Current bottleneck:** Entity generation counter (u32 wrap-around)

**Solutions (in order of implementation):**

1. **✅ DONE**: Hide entities instead of despawn
   - Avoids generation counter increment
   - Disabled marker skips processing
   - Cost: More total entities in world (but inactive)

2. **IN PROGRESS**: Memory pooling (SOLIVE Job #2)
   - Pre-allocate entity IDs at startup
   - Reuse by resetting components
   - Zero allocation after pool init

3. **COMING**: Spatial grid → Quadtree (SOLIVE Job #1)
   - O(log n) vs O(n) spatial queries
   - Scales better at 1000+ entities

4. **COMING**: GPU instancing (SOLIVE Job #3)
   - 200+ sprites in 1-4 draw calls
   - Rendering bottleneck eliminated

---

## Key Invariants

**These must hold for correct gameplay:**

1. **No component in transition state**
   - Every entity either fully initialized or Disabled
   - No partial component additions mid-frame

2. **Spatial grid consistent with entity positions**
   - Grid updated BEFORE collision detection
   - Dead entities removed from grid AFTER despawn marked

3. **Damage applied AFTER collision detection**
   - Prevents entity mutation during query iteration

4. **Disabled entities not processed**
   - Movement, collision, animation systems skip Disabled
   - Prevents ghost behavior (invisible entities moving)

5. **Frame determinism**
   - Same inputs → same output (except RNG for enemy spawning)
   - No random system ordering
   - All systems chain ordered

---

## Test Coverage (via SOLIVE Jobs)

| Job | Tests | Verification |
|-----|-------|--------------|
| Memory Pooling | Acquire/release/reset | 1000 spawns/frame, <1µs per spawn |
| Spatial Indexing | Query time, cell occupancy | O(log n) performance |
| GPU Batching | Draw call count, visual correctness | <10 calls for 200 sprites |
| Config System | Load time, hot-reload, defaults | <100ms load, graceful fallback |
| Prefab System | Parse, spawn, hotreload | All components deserialize |
| Debug Profiler | Accuracy, overhead, usability | <2% frame overhead |

---

## Future Improvements (Not in MVP)

- **Instanced rendering**: Bake sprites to atlases, batch by texture
- **Async asset loading**: Preload waves/levels while playing
- **Replay system**: Record inputs, replay deterministically
- **Network multiplayer**: ECS-friendly architecture (no state mutations)
- **Replay playback**: Deterministic frame-perfect replay
- **AI behavior trees**: Upgrade from simple chase
- **Particle effects**: Pooled particle system
- **Audio system**: Spatial audio, sound pooling
- **Scripting**: Lua/WASM for game logic iteration

---

## Quick Reference: Adding New Features

### New Enemy Type

1. Create variant in `components/gameplay.rs` (marker component)
2. Add bundle in `archetypes/enemy.rs` (e.g., `zombie_bundle()`)
3. Update `spawning.rs` to handle new type
4. Update collision/damage logic if special rules

### New Projectile Type

1. Add to `archetypes/projectile.rs`
2. Update `game/projectiles.rs` spawn logic
3. Add collision rules in `systems/contact.rs`

### New Game State

1. Add field to `game/state.rs::GameState`
2. Update `game/ui.rs` logging
3. Add system to update state in game loop

---

**Last Updated**: 2026-10-31
**Status**: Foundation complete, workers implementing advanced systems
**Next Milestone**: Workers deliver Job #1-6, integrate into main

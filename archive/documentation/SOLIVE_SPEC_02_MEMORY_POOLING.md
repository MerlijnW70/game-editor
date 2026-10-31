# Task: Implement Memory Pooling & Entity Recycling

## Objective
Build an **object pool system** for high-churn entities (bullets, particles, enemies) to eliminate spawn/despawn overhead and reduce GC pressure.

## Why This Matters
- Current: spawn_batch + despawn creates 60,000 ops/sec at high entity churn
- Target: Pre-allocate pools, recycle entities → 0 allocations after pool init
- Proves engine can handle 1000+ projectiles/frame

## Technical Requirements

### 1. Pool Manager Resource
- **Structure**: `struct PoolManager { enemy_pool: Vec<Entity>, projectile_pool: Vec<Entity>, ... }`
- **Pool size**: Configurable per entity type (default: 50 enemies, 200 projectiles)
- **Operations**:
  - `fn acquire(&mut self, pool_type: PoolType) -> Entity` → grab from pool or spawn
  - `fn release(&mut self, entity: Entity, pool_type: PoolType)` → return to pool
  - `fn reset_entity(&mut self, entity: Entity, components: Bundle)` → clear + repopulate

### 2. Pooling Strategy
- **Pre-allocate**: On app startup, spawn max pool size, hide them (disabled, out of bounds)
- **Acquire**: Move entity to active position, re-enable, insert new components
- **Release**: Clear components, hide entity, return to pool
- **No spawning after init**: All entity IDs static, only component state changes

### 3. Integration Points
- **Enemy spawning**: Use `pool.acquire(PoolType::Enemy)` instead of `commands.spawn(...)`
- **Projectile spawning**: Use `pool.acquire(PoolType::Projectile)` instead of spawn
- **Despawn**: Use `pool.release(entity, type)` instead of `commands.entity(e).despawn()`
- **Systems**: Create `pool_maintenance_system` to reset released entities

### 4. Performance Targets
- **Spawn (pool)**: < 1µs per entity (vs 100µs for spawn_batch)
- **Frame with 1000 projectiles spawned**: < 5ms total (vs 50ms without pooling)
- **Memory**: Pre-allocated once, no growth after init

### 5. Example & Tests
- `examples/pooling_demo.rs`: Spawn/release 1000 projectiles per frame, measure FPS
- Unit tests: acquire, release, reset operations
- Stress test: 5000 simultaneous entities with 500 spawns/frame

## Deliverable Format
- `src/resources/pool_manager.rs` → PoolManager struct + methods
- `src/systems/pooling.rs` → pool maintenance system
- Update `src/plugin.rs` to register PoolManager resource
- Example: `examples/pooling_demo.rs` with 1000 projectiles
- Tests: `tests/pooling.rs` covering acquire, release, reset
- Documentation: `POOLING_DESIGN.md`

## Acceptance Criteria
1. PoolManager compiles, no warnings
2. All tests pass (acquire, release, reset)
3. Pooling demo: 1000 projectiles/frame at 60+ FPS
4. Spawn time measured < 1µs per entity (vs 100µs baseline)
5. Memory pre-allocated, no runtime allocations
6. Works with collision system (pooled entities collide correctly)
7. Git commit with clear message

## Payment
**0.07 SOL**

## Timeline
- **Estimated**: 5-7 hours
- **Deadline**: 7 days from job posting

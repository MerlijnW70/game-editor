# SOLIVE Job: Pool Component Reset

**Budget:** 0.06 SOL  
**Deadline:** 7 days  
**Effort:** 2-3 hours  
**Status:** Phase 2 - Ready after Phase 1 complete  
**Depends On:** Job #1 (validation framework)

---

## Overview

Object pool reuses entities via `Disabled` marker, but when entities are re-acquired, their old components are not cleared. This causes stale data (old health values, old velocities, old colliders) to persist on recycled entities.

---

## Problem

Current code in `src/pool.rs`:
```rust
pub fn acquire(&mut self, pool_type: PoolType) -> Option<Entity> {
    if let Some(entity) = self.disabled_enemies.pop() {
        commands.entity(entity).remove::<Disabled>();  // ❌ No component cleanup!
        return Some(entity);
    }
    None
}
```

When entity is reused:
- Old `Health(50)` stays → entity spawns with previous health
- Old `Velocity(Vec2::new(-100, 0))` stays → entity moves in old direction
- Old `Transform2D` stays → entity appears at old position

Result: Flickering entities, wrong movement vectors, zombies from previous life.

---

## Requirements

1. **Add component cleanup on acquire** (src/pool.rs)
   ```rust
   pub fn acquire(&mut self, pool_type: PoolType) -> Option<Entity> {
       if let Some(entity) = self.disabled_enemies.pop() {
           // NEW: Reset to clean state
           commands.entity(entity)
               .remove::<Disabled>()
               .remove::<Health>()
               .remove::<Velocity>()
               .remove::<Transform2D>()
               .remove::<Collider>()
               .remove::<Sprite>();  // Any pooled components
           
           return Some(entity);
       }
       None
   }
   ```

2. **Create component reset trait** (src/pool.rs)
   ```rust
   pub trait PoolReset {
       fn reset(&mut self);
   }
   
   impl PoolReset for Health {
       fn reset(&mut self) {
           *self = Health::new(100.0);
       }
   }
   
   impl PoolReset for Velocity {
       fn reset(&mut self) {
           *self = Velocity(Vec2::ZERO);
       }
   }
   
   // Similar for Transform2D, Collider, Sprite
   ```

3. **Add test for stale component cleanup** (tests/pool.rs)
   ```rust
   #[test]
   fn test_pool_reset_clears_stale_components() {
       let mut pool = EntityPool::new(100);
       let e1 = Entity::from_raw(1);
       
       // Simulate first use: entity with old health/velocity
       pool.set_entity_health(e1, 25.0);
       pool.set_entity_velocity(e1, Vec2::new(-50.0, 0.0));
       
       // Return to pool
       pool.release(e1);
       
       // Acquire again
       let e_reused = pool.acquire(PoolType::Enemy).unwrap();
       
       // Should have clean state (not old health/velocity)
       let health = get_health(e_reused);
       let velocity = get_velocity(e_reused);
       
       assert_eq!(health, 100.0, "Health should be reset to default");
       assert_eq!(velocity, Vec2::ZERO, "Velocity should be reset to zero");
   }
   
   #[test]
   fn test_pool_reset_multiple_times() {
       // Cycle entity 10× through pool
       // Verify stale state never persists
       for _ in 0..10 {
           let entity = pool.acquire(PoolType::Enemy).unwrap();
           assert_eq!(entity.health, 100.0);
           
           // Damage it
           apply_damage(entity, 50.0);
           assert_eq!(entity.health, 50.0);
           
           // Return to pool
           pool.release(entity);
       }
   }
   
   #[test]
   fn test_no_stale_data_in_integrated_demo() {
       // Run integrated_demo for 1000 frames
       // All respawned enemies should have correct health/velocity
       // No entity should have cross-life contamination
   }
   ```

4. **Add safety documentation** (src/pool.rs)
   ```rust
   /// Pool entities are automatically reset on acquire.
   /// 
   /// Components reset:
   /// - Health → 100.0
   /// - Velocity → (0, 0)
   /// - Transform2D → (0, 0, 0)
   /// - Sprite → default visibility
   ///
   /// Do not rely on component state persisting across pool cycles.
   pub fn acquire(&mut self, pool_type: PoolType) -> Option<Entity>
   ```

---

## Acceptance Criteria

✅ `acquire()` removes all stale components  
✅ `Health` reset to default (100.0)  
✅ `Velocity` reset to zero  
✅ `Transform2D` reset to origin  
✅ All 3 test cases pass  
✅ No stale data in integrated_demo  
✅ No flickering or wrong-direction entities after respawn  
✅ Compiles with no warnings  

---

## Verification Command

```bash
cargo test pool_reset_clears_stale_components
cargo test pool_reset_multiple_times
cargo test no_stale_data_in_integrated_demo
cargo run --release --example integrated_demo -- --stress-test
# Watch: respawned enemies should have correct health bar color
# Watch: respawned enemies should move toward player (not old direction)
```

---

## Judge Check

```bash
./verify_solive_delivery.ps1 JOB_POOL_RESET
```

Expected: ✅ No stale components, entities respawn clean, no visual glitches


# SOLIVE Job: Performance Optimization & Metrics Fixes

**Budget:** 0.12 SOL  
**Deadline:** 7 days  
**Effort:** 3-4 hours  
**Status:** Open for claims

---

## Spec

Fix O(n) performance bottlenecks and metrics overflow issues:
1. Entity count queries happening every frame (60 fps × 8192 entities = millions of queries)
2. Metrics accumulation overflows after ~3 days of continuous play
3. Pool exhaustion returns None with no indication

### Problem 1: O(n) Entity Count Queries

Current code queries all entities every frame:
```rust
// spawning.rs:55
if all_entities.iter().count() >= MAX_ENTITIES {  // ❌ O(n) every frame!
    return;
}

// game/projectiles.rs - similar issue
let active_projectiles = projectiles.iter().count();  // ❌ O(n) every frame!
```

At 8K entities × 60 fps = 480K entity iterations per second!

### Problem 2: Metrics Overflow

```rust
// debug/profiler.rs
pub total_frame_time_ms: f32;  // ❌ Accumulates forever
// After ~3 days: 60 fps × 3 days = 15.5M frames × 16.67ms = 260 billion ms = overflow!
```

### Requirements

1. **Cache active enemy count in EnemySpawner** (src/systems/spawning.rs)
   ```rust
   #[derive(Resource, Debug, Clone)]
   pub struct EnemySpawner {
       pub target_count: usize,
       pub spawn_radius: f32,
       pub max_health: f32,
       pub speed: f32,
       pub max_per_frame: usize,
       pub active_enemy_count: usize,  // NEW: Cache instead of querying
       rng: u64,
   }
   
   impl EnemySpawner {
       pub fn on_enemy_spawned(&mut self) {
           self.active_enemy_count += 1;
       }
       
       pub fn on_enemy_despawned(&mut self) {
           self.active_enemy_count = self.active_enemy_count.saturating_sub(1);
       }
   }
   ```

2. **Update EnemySpawner when enemies spawn/despawn** (src/systems/spawning.rs & despawn.rs)
   ```rust
   // spawning.rs
   pub fn spawn_enemies(
       mut commands: Commands,
       mut spawner: ResMut<EnemySpawner>,
       // ... other params
   ) {
       if spawner.active_enemy_count >= 8192 {  // ✅ O(1) check
           return;
       }
       
       // ... spawn logic
       spawner.active_enemy_count += 1;  // ✅ Update cache
   }
   
   // despawn.rs
   pub fn despawn_dead(
       mut commands: Commands,
       mut spawner: ResMut<EnemySpawner>,
       query: Query<Entity, With<Dead>>,
   ) {
       for entity in query.iter() {
           commands.entity(entity).insert(Disabled);
           spawner.on_enemy_despawned();  // ✅ Update cache
       }
   }
   ```

3. **Cache projectile count in FireRateLimiter** (src/game/projectiles.rs)
   ```rust
   #[derive(Resource, Default, Debug, Clone)]
   pub struct FireRateLimiter {
       pub fire_rate: f32,
       pub last_fire_frame: u64,
       pub active_projectile_count: usize,  // NEW: Cache
   }
   
   // Update when spawning/despawning projectiles
   ```

4. **Fix metrics accumulation** (src/debug/profiler.rs)
   ```rust
   impl DebugProfiler {
       pub fn average_frame_time_ms(&self) -> f32 {
           if self.frame_history.is_empty() { return 0.0; }
           
           // ✅ Calculate from history, not accumulation
           self.frame_history
               .iter()
               .map(|m| m.delta_time_ms)
               .sum::<f32>() / self.frame_history.len() as f32
       }
       
       pub fn reset_stats(&mut self) {
           self.frame_history.clear();
           self.total_frame_time_ms = 0.0;  // ✅ Reset accumulator
           self.peak_frame_time_ms = 0.0;
           self.min_frame_time_ms = f32::MAX;
       }
   }
   ```

5. **Log pool exhaustion instead of silent fallback** (src/systems/spawning.rs)
   ```rust
   for _ in 0..count {
       if let Some(entity) = pool.acquire(PoolType::Enemy) {
           // Reuse entity
       } else {
           warn!("Enemy pool exhausted, falling back to spawn_batch");
           fallback_spawns.push(enemy_bundle(...));
       }
   }
   ```

6. **Add tests** (tests/performance.rs)
   ```rust
   #[test]
   fn test_enemy_count_cache() {
       let mut spawner = EnemySpawner::new(100);
       assert_eq!(spawner.active_enemy_count, 0);
       
       spawner.on_enemy_spawned();
       assert_eq!(spawner.active_enemy_count, 1);
       
       spawner.on_enemy_despawned();
       assert_eq!(spawner.active_enemy_count, 0);
   }
   
   #[test]
   fn test_metrics_no_overflow() {
       let mut profiler = DebugProfiler::new();
       
       // Simulate 3 days of continuous play
       for _ in 0..15_500_000 {
           let metrics = FrameMetrics {
               delta_time_ms: 16.67,
               ..Default::default()
           };
           profiler.record_frame(metrics);
       }
       
       // Should not panic or overflow
       let avg = profiler.average_frame_time_ms();
       assert!(avg > 0.0 && avg < 1000.0);  // Reasonable range
   }
   
   #[test]
   fn test_cache_better_than_query() {
       // Benchmark: cache lookup should be <1µs
       // Query should be >100µs at scale
       // Use std::time::Instant to verify
   }
   ```

---

## Acceptance Criteria

✅ `EnemySpawner` has `active_enemy_count` field (cached, not queried)  
✅ `on_enemy_spawned()` and `on_enemy_despawned()` methods  
✅ `spawn_enemies()` uses cache instead of query  
✅ `despawn_dead()` updates cache  
✅ Metrics use rolling average instead of accumulation  
✅ `reset_stats()` clears accumulator  
✅ Pool exhaustion logged (not silent)  
✅ All 3 test cases pass  
✅ Spawning at 8K entities no longer freezes (remove O(n) query)  
✅ Metrics stable after 3 days of play  
✅ Compiles with no warnings  

---

## Verification Command

```bash
cargo test enemy_count_cache
cargo test metrics_no_overflow
cargo test cache_better_than_query
cargo run --release --example integrated_demo -- --frames 100000
# Should NOT see frame time spikes from entity count queries
```

---

## Judge Check

```bash
./verify_solive_delivery.ps1 JOB_PERF_FIX
```

Expected: ✅ O(n) queries replaced with O(1) caches, metrics stable, no overflow

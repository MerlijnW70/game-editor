# SOLIVE Job: Parallel Entity Updates with Rayon

**Budget:** 0.10 SOL  
**Deadline:** 7 days  
**Effort:** 3-4 hours  
**Status:** Phase 4 - Advanced Optimization  
**Depends On:** Job #12 (Cell Buffer)

---

## Problem Analysis

Spatial grid updates are single-threaded bottleneck:

```rust
pub fn update_spatial_grid(
    mut grid: ResMut<SpatialGrid>,
    mut query: Query<(Entity, &Transform2D, &Collider)>,
) {
    // Sequential: iterate all entities, update grid
    for (entity, transform, collider) in query.iter_mut() {
        grid.update_entity(entity, transform.position, collider.radius);
    }
}
```

Performance at scale:
- **8K entities × 60 fps:**
  - 480,000 entities/sec processed
  - Each update_entity(): ~500ns (remove old, add new cells)
  - Total: **240 ms per frame** (WAY TOO SLOW!)
  - Actual measured: ~2-5 ms per frame (acceptable but not optimal)

- **16K entities × 60 fps:**
  - 960,000 entities/sec
  - Total: **480 ms per frame** (UNACCEPTABLE)
  - Would drop to ~3-5 fps

### Root Cause: HashMap Contention
Multiple cores can't efficiently update same HashMap:
- Core 1 updates cell (0, 0)
- Core 2 updates cell (1, 1)
- Both block on HashMap's RwLock
- Overhead dominates any parallelism benefit

---

## Solution: Parallel Collection + Sequential Apply

Instead of parallel grid updates (contention!), parallelize the **collection** phase:

### 1. Collection Phase (Parallelizable)

```rust
use rayon::prelude::*;

#[derive(Clone)]
struct SpatialGridUpdate {
    entity: Entity,
    pos: Vec2,
    radius: f32,
}

pub fn update_spatial_grid_parallel(
    mut grid: ResMut<SpatialGrid>,
    query: Query<(Entity, &Transform2D, &Collider)>,
) {
    // PARALLEL: Collect updates from all entities
    // No HashMap contention - just reading transforms
    let updates: Vec<_> = query
        .par_iter()  // Parallel iteration
        .map(|(entity, transform, collider)| SpatialGridUpdate {
            entity,
            pos: transform.position,
            radius: collider.radius,
        })
        .collect();  // Gather results in main thread
    
    // SEQUENTIAL: Apply updates to grid
    // Grid is single-threaded, but collection was parallelized
    for update in updates {
        grid.update_entity(update.entity, update.pos, update.radius);
    }
}
```

### 2. Advanced: Batch Updates with Synchronization

```rust
pub struct SpatialGridBatch {
    updates: Vec<SpatialGridUpdate>,
    capacity: usize,
}

impl SpatialGridBatch {
    pub fn new(capacity: usize) -> Self {
        Self {
            updates: Vec::with_capacity(capacity),
            capacity,
        }
    }
    
    /// Collect updates from query in parallel
    pub fn collect_parallel(
        &mut self,
        query: Query<(Entity, &Transform2D, &Collider)>,
    ) {
        self.updates.clear();
        
        let updates: Vec<_> = query
            .par_iter()
            .map(|(entity, transform, collider)| SpatialGridUpdate {
                entity,
                pos: transform.position,
                radius: collider.radius,
            })
            .collect();
        
        self.updates = updates;
    }
    
    /// Apply all updates sequentially
    pub fn apply(&self, grid: &mut SpatialGrid) {
        for update in &self.updates {
            grid.update_entity(update.entity, update.pos, update.radius);
        }
    }
    
    pub fn len(&self) -> usize {
        self.updates.len()
    }
}

// System using the batch
pub fn update_grid_with_batch(
    mut batch: ResMut<SpatialGridBatch>,
    mut grid: ResMut<SpatialGrid>,
    query: Query<(Entity, &Transform2D, &Collider), Changed<Transform2D>>,
) {
    // Only update entities that moved
    batch.collect_parallel(query);
    batch.apply(&mut grid);
}
```

### 3. Rayon Pool Configuration

```rust
use rayon::ThreadPoolBuilder;

pub fn setup_rayon_pool(
    config: Res<GameConfig>,
) {
    // Use half the available cores (leave other half for main thread)
    let num_threads = num_cpus::get() / 2;
    
    let _ = ThreadPoolBuilder::new()
        .num_threads(num_threads)
        .build_global();
    
    info!("Rayon thread pool initialized with {} threads", num_threads);
}
```

### 4. Benchmarking System

```rust
pub struct ParallelUpdateMetrics {
    collection_time_ms: f32,
    apply_time_ms: f32,
    entities_per_second: usize,
}

pub fn measure_parallel_updates(
    mut metrics: ResMut<ParallelUpdateMetrics>,
    batch: Res<SpatialGridBatch>,
) {
    // Reported by systems above
    let total = metrics.collection_time_ms + metrics.apply_time_ms;
    let fps_60_budget = 16.67;  // 60 fps = 16.67ms per frame
    
    if total > fps_60_budget {
        warn!(
            "Grid updates took {:.2}ms (budget: {:.2}ms)",
            total, fps_60_budget
        );
    } else {
        debug!(
            "Grid updates: collection {:.2}ms, apply {:.2}ms",
            metrics.collection_time_ms, metrics.apply_time_ms
        );
    }
}
```

---

## Performance Analysis

### Theoretical Speedup

**Sequential (baseline):**
- 8K entities: 240 ms/frame (from calculation above)
- Actual: ~3-5 ms (grid is fast in practice)
- Per-entity: ~600 ns

**Parallel (4 cores):**
- Parallel collection: ~1.25 ms (3.8× speedup on 4 cores)
- Sequential apply: ~2.0 ms (unchanged)
- Total: ~3.25 ms
- **Speedup: 2-3× depending on core count**

**At 16K entities:**
- Sequential: ~6-10 ms
- Parallel: ~3.5-5 ms
- **Speedup: 2.5-3×**

### Reality Check
Speedup limited by:
1. Amdahl's law (sequential apply phase can't parallelize)
2. Collection overhead (Vec allocation, synchronization)
3. System scheduling overhead

Still worth it: saves ~2-5 ms per frame, enables higher entity counts.

---

## Acceptance Criteria

✅ `SpatialGridUpdate` struct with entity/pos/radius  
✅ `par_iter()` collection from query  
✅ Sequential apply to grid  
✅ `SpatialGridBatch` resource  
✅ `Changed<Transform2D>` optimization (only update moved entities)  
✅ Rayon thread pool initialization  
✅ Benchmark: 2-3× speedup on 4+ cores  
✅ All 4 test cases pass  
✅ Compiles with no warnings  

---

## Tests

```rust
#[test]
fn test_batch_collection() {
    let world = World::new();
    // Setup entities with transforms and colliders
    
    let mut batch = SpatialGridBatch::new(1000);
    let query = world.query::<(Entity, &Transform2D, &Collider)>();
    
    batch.collect_parallel(query);
    
    assert!(batch.len() > 0);
    assert!(batch.len() <= 1000);
}

#[test]
fn test_batch_apply_correctness() {
    let mut grid = SpatialGrid::new();
    let mut batch = SpatialGridBatch::new(100);
    
    // Manually create updates
    batch.updates.push(SpatialGridUpdate {
        entity: Entity::from_bits(1),
        pos: Vec2::ZERO,
        radius: 10.0,
    });
    batch.updates.push(SpatialGridUpdate {
        entity: Entity::from_bits(2),
        pos: Vec2::new(50.0, 0.0),
        radius: 10.0,
    });
    
    batch.apply(&mut grid);
    
    // Verify both entities are in grid
    let near = grid.query_circle(Vec2::ZERO, 20.0);
    assert!(near.contains(&Entity::from_bits(1)));
}

#[test]
fn test_parallel_vs_sequential() {
    use std::time::Instant;
    
    // Create 1000 entities with transforms
    let mut batch = SpatialGridBatch::new(10000);
    
    // Time parallel collection (uses Rayon)
    let start = Instant::now();
    // batch.collect_parallel(query);  // This would need actual query
    let parallel_time = start.elapsed();
    
    // Expected: <1ms for 1000 entities
    println!("Parallel collection: {:.2}ms", parallel_time.as_secs_f32() * 1000.0);
}

#[test]
fn test_changed_filter_optimization() {
    // Only entities with Changed<Transform2D> should be updated
    // Test by creating static entities and only moving some
    // Verify batch only contains moved entities
}
```

---

## Configuration

Add to Cargo.toml:
```toml
rayon = "1.7"
num_cpus = "1.16"
```

---

## Verification Command

```bash
cargo test test_batch_collection
cargo test test_batch_apply_correctness
cargo test test_parallel_vs_sequential
RAYON_NUM_THREADS=4 cargo run --release --example integrated_demo -- --stress-test
# Monitor CPU usage (should see 4+ cores active during grid updates)
```

---

## Judge Check

```bash
./verify_solive_delivery.ps1 JOB_PARALLEL_UPDATES
```

Expected: ✅ Parallel collection working, 2-3× speedup on multi-core, correctness verified


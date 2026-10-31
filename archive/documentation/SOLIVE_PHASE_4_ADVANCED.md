# Phase 4: Advanced Micro-Optimizations & Features

**Status:** Optional, planned after Phase 3  
**Total Budget:** 0.40+ SOL  
**Focus:** Cutting-edge performance, production polish, nice-to-have features

---

## Overview

Phase 4 represents ambitious optimizations beyond the critical path. These are **not blocking production** but enable:
- 120 fps at 16K entities (instead of 60 fps)
- Advanced visual features
- Production-grade tooling

| # | Job | Budget | Focus | Type |
|----|-----|--------|-------|------|
| 11 | Allocation Pool for Queries | 0.08 SOL | Reuse Vec allocations | Perf |
| 12 | Cell Buffer Pre-allocation | 0.07 SOL | SIMD grid calculations | Perf |
| 13 | Hot-reload Config | 0.10 SOL | Live config changes | Feature |
| 14 | Advanced Profiler | 0.08 SOL | Production-grade metrics | Feature |
| 15 | Parallel Entity Updates | 0.10 SOL | Multi-threaded grid updates | Perf |

---

## Job 11: Allocation Pool for Queries (0.08 SOL)

### Problem

Current code allocates new `Vec<Entity>` for every query:

```rust
pub fn query_circle(&self, pos: Vec2, radius: f32) -> Vec<Entity> {
    let cells = Self::cells_for_circle(pos, radius);  // ❌ New Vec allocated
    let mut result = Vec::new();  // ❌ New Vec allocated
    // ... populate result
    result  // Return owned Vec
}
```

At 60 fps × 100 queries per frame = 6000 allocations per second = pressure on allocator.

### Solution

```rust
pub struct QueryBufferPool {
    pools: [VecDeque<Vec<Entity>>; 8],  // Pre-allocated buffers (8 sizes)
    active: usize,
}

impl QueryBufferPool {
    pub fn acquire(&mut self, capacity: usize) -> BufferGuard {
        // Return pre-allocated Vec from pool
        // Allocate only if pool empty
    }
    
    pub fn release(&mut self, buffer: Vec<Entity>) {
        // Return to pool for reuse
    }
}

// Usage:
let mut buffer = pool.acquire(100);
buffer.clear();
// ... populate
queries.push(QueryResult::Borrowed(buffer));
// Auto-return to pool when dropped
```

### Deliverables
- `QueryBufferPool` resource
- `BufferGuard` wrapper (RAII)
- Integrated into collision queries
- Benchmark: <1µs acquire vs 10µs alloc
- 3 unit tests

### Time: 3-4 hours

---

## Job 12: Cell Buffer Pre-allocation (0.07 SOL)

### Problem

Grid calculations allocate new Vec for every `cells_for_circle()` call:

```rust
fn cells_for_circle(pos: Vec2, radius: f32) -> Vec<(i32, i32)> {
    let mut cells = Vec::new();  // ❌ New allocation
    for gx in min_x..=max_x {
        for gy in min_y..=max_y {
            cells.push((gx, gy));
        }
    }
    cells
}
```

Worst case (large radius): 400 cells × 60 fps = 24,000 allocations/sec.

### Solution

```rust
pub struct GridCalculator {
    cell_buffer: Vec<(i32, i32)>,  // Pre-allocated, reused
}

impl GridCalculator {
    #[inline]
    pub fn cells_for_circle(&mut self, pos: Vec2, radius: f32) -> &[(i32, i32)] {
        self.cell_buffer.clear();
        
        let min_x = ((pos.x - radius) / GRID_CELL_SIZE).floor() as i32;
        let max_x = ((pos.x + radius) / GRID_CELL_SIZE).floor() as i32;
        let min_y = ((pos.y - radius) / GRID_CELL_SIZE).floor() as i32;
        let max_y = ((pos.y + radius) / GRID_CELL_SIZE).floor() as i32;
        
        for gx in min_x..=max_x {
            for gy in min_y..=max_y {
                self.cell_buffer.push((gx, gy));
            }
        }
        
        &self.cell_buffer  // Return slice, no copy
    }
}
```

### Deliverables
- `GridCalculator` resource
- Pre-allocated 1024-cell buffer
- SIMD-friendly layout
- Benchmark: Zero allocations in hot path
- 4 unit tests (buffer reuse, correctness)

### Time: 2-3 hours

---

## Job 13: Hot-reload Config (0.10 SOL)

### Problem

Config changes require game restart. Can't tweak values during play.

### Solution

```rust
pub struct HotReloadConfig {
    file_path: PathBuf,
    last_modified: SystemTime,
    config: GameConfig,
}

impl HotReloadConfig {
    pub fn try_reload(&mut self) -> Result<bool, ConfigError> {
        if let Ok(metadata) = fs::metadata(&self.file_path) {
            if metadata.modified()? > self.last_modified {
                self.config = GameConfig::load(&self.file_path)?;
                self.last_modified = metadata.modified()?;
                return Ok(true);  // Changed
            }
        }
        Ok(false)  // No change
    }
}

// System: check for changes every frame
pub fn reload_config(
    mut config: ResMut<HotReloadConfig>,
    mut events: EventWriter<ConfigReloadedEvent>,
) {
    if let Ok(changed) = config.try_reload() {
        if changed {
            info!("Config reloaded from disk");
            events.send(ConfigReloadedEvent);
        }
    }
}
```

### Deliverables
- `HotReloadConfig` resource
- File watching system
- `ConfigReloadedEvent`
- Integration with ECS (listen for event)
- 5 unit tests (file changes, format errors, etc.)

### Time: 3-4 hours

---

## Job 14: Advanced Profiler (0.08 SOL)

### Problem

Current profiler is basic. Production needs:
- Per-system timing (spawning, collision, rendering, etc.)
- Allocation tracking
- Cache miss estimation
- Heatmap visualization data

### Solution

```rust
pub struct AdvancedProfiler {
    // Per-system timing
    system_timings: HashMap<SystemId, RollingAverage<f32>>,
    
    // Allocation stats
    allocation_counts: HashMap<String, usize>,
    
    // Memory heatmap (for visualization)
    frame_heatmap: Vec<f32>,  // Time spent per 1KB bucket
}

impl AdvancedProfiler {
    pub fn record_system(&mut self, system_id: SystemId, duration_ms: f32) {
        self.system_timings
            .entry(system_id)
            .or_insert_with(RollingAverage::new)
            .record(duration_ms);
    }
    
    pub fn report(&self) -> ProfilingReport {
        let mut systems: Vec<_> = self.system_timings.iter()
            .map(|(id, avg)| (id, avg.average()))
            .collect();
        systems.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        
        ProfilingReport {
            top_systems: systems.into_iter().take(10).collect(),
            total_frame_time: self.system_timings.values()
                .map(|avg| avg.average())
                .sum(),
        }
    }
}
```

### Deliverables
- `AdvancedProfiler` resource
- Per-system timing (10+ systems)
- Allocation counter
- `ProfilingReport` with top-N systems
- 4 unit tests

### Time: 3-4 hours

---

## Job 15: Parallel Entity Updates (0.10 SOL)

### Problem

Spatial grid updates are sequential. At 8K entities:
- 8000 entities × 60 fps = 480K updates/sec
- Single-threaded bottleneck

### Solution

```rust
pub fn update_spatial_grid_parallel(
    mut grid: ResMut<SpatialGrid>,
    mut query: Query<(Entity, &Transform2D, &Collider)>,
) {
    // Split work across 4 threads (configurable)
    let updates: Vec<_> = query
        .par_iter()
        .map(|(entity, transform, collider)| {
            SpatialGridUpdate {
                entity,
                pos: transform.position,
                radius: collider.radius,
            }
        })
        .collect();
    
    // Apply updates sequentially (to maintain HashMap consistency)
    for update in updates {
        grid.update_entity(update.entity, update.pos, update.radius);
    }
}
```

### Deliverables
- Parallel query collection using `par_iter()`
- Batch update processing
- Configurable thread count
- Benchmark: 2-3× speedup on 8K entities
- 4 unit tests (correctness, speedup)

### Time: 3-4 hours

---

## Implementation Order

**Recommended sequence:**
1. Job 11 (Query Pool) — Easy win, high impact
2. Job 12 (Cell Buffer) — Builds on 11
3. Job 15 (Parallel) — Requires Job 12
4. Job 14 (Profiler) — Can run anytime
5. Job 13 (Hot-reload) — Nice-to-have

---

## Expected Performance Gains

| Job | Bottleneck | Current | After | Gain |
|-----|-----------|---------|-------|------|
| 11 | Query allocation | 10µs | <1µs | 10× |
| 12 | Cell buffer alloc | 5µs | 0µs | ∞ |
| 15 | Sequential update | 480K upd/sec | 1.2M upd/sec | 2.5× |
| 11+12+15 | Combined | 60 fps @ 8K | 120 fps @ 8K | **2× FPS** |

---

## Timeline to 120 fps

| Phase | Status | FPS @ 8K | Date |
|-------|--------|----------|------|
| Current | Live | 60 fps | Today |
| Phase 1-3 | ~7 days | 60 fps (stable) | Day 8 |
| Phase 4 | Optional | 120 fps | Day 12-14 |

---

## Budget & ROI

**Phase 4 Total:** 0.40 SOL  
**Total Codebase (1-4):** ~1.4 SOL  

**Engineering Value:**
- 28-45 hours of expert optimization
- From foundation (65%) → production (95%) → high-perf (120 fps)
- Comparable to 1-2 months solo engineering

---

## Optional: Phase 5+ (Beyond Production)

| Phase | Focus | Budget | Timeline |
|-------|-------|--------|----------|
| Phase 5 | Physics simulation | 0.15 SOL | Day 15+ |
| Phase 6 | Custom shaders | 0.12 SOL | Day 18+ |
| Phase 7 | Advanced AI | 0.20 SOL | Day 21+ |

---

## Next Steps

1. **Complete Phase 1-3** (10 days)
2. **Measure performance** at end of Phase 3
3. **Decide on Phase 4** based on needs:
   - Need 120 fps? → Implement Jobs 11-15
   - Happy with 60 fps stable? → Ship Phase 3
4. **Plan Phase 5+** if needed


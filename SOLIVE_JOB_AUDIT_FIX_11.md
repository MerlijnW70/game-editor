# SOLIVE Job: Query Result Allocation Pooling

**Budget:** 0.08 SOL  
**Deadline:** 7 days  
**Effort:** 3-4 hours  
**Status:** Phase 4 - Advanced Optimization  
**Blocks:** Job #15 (Parallel Updates)

---

## Problem Analysis

Spatial queries allocate new `Vec<Entity>` for every query call:

```rust
pub fn query_circle(&self, pos: Vec2, radius: f32) -> Vec<Entity> {
    let cells = Self::cells_for_circle(pos, radius);  // ❌ Alloc #1
    let mut result = Vec::new();  // ❌ Alloc #2
    let mut seen = HashSet::new();  // ❌ Alloc #3
    
    for cell in cells {
        if let Some(entities) = self.cells.get(&cell) {
            for &entity in entities {
                if seen.insert(entity) {
                    result.push(entity);
                }
            }
        }
    }
    result  // 3-4 allocations per query!
}
```

At production scale:
- 60 fps × 100 collision queries per frame = 6000 queries/sec
- 3-4 allocations per query = 18,000-24,000 allocations/sec
- Each allocation: 200-500 ns (modern allocator)
- Total overhead: **3-12 ms per frame** at high entity count

---

## Solution Architecture

### 1. Query Buffer Pool

```rust
use std::collections::VecDeque;

/// Pre-allocated buffers for reuse across frames
pub struct QueryBufferPool {
    /// Separate pools for different capacities (power-of-2 buckets)
    pools: [VecDeque<Vec<Entity>>; 8],  // 8, 16, 32, 64, 128, 256, 512, 1024+
    stats: PoolStats,
}

#[derive(Debug, Clone)]
pub struct PoolStats {
    pub allocations: usize,
    pub reuses: usize,
    pub peak_pool_size: usize,
}

impl QueryBufferPool {
    pub fn new() -> Self {
        Self {
            pools: [
                VecDeque::with_capacity(8),    // 8 buffers
                VecDeque::with_capacity(8),
                VecDeque::with_capacity(8),
                VecDeque::with_capacity(8),
                VecDeque::with_capacity(8),
                VecDeque::with_capacity(4),
                VecDeque::with_capacity(4),
                VecDeque::with_capacity(2),
            ],
            stats: PoolStats::default(),
        }
    }
    
    /// Get a buffer suitable for capacity hint
    /// Returns either a pooled buffer or allocates new one
    pub fn acquire(&mut self, hint_capacity: usize) -> BufferGuard {
        let bucket = Self::bucket_for_capacity(hint_capacity);
        
        if let Some(mut buffer) = self.pools[bucket].pop_front() {
            buffer.clear();  // Reuse!
            self.stats.reuses += 1;
            BufferGuard::new(buffer, self, bucket)
        } else {
            let capacity = 1 << (bucket + 3);  // Compute actual capacity
            self.stats.allocations += 1;
            BufferGuard::new(Vec::with_capacity(capacity), self, bucket)
        }
    }
    
    /// Return buffer to pool for reuse
    pub(crate) fn release(&mut self, buffer: Vec<Entity>, bucket: usize) {
        if self.pools[bucket].len() < 8 {  // Keep pool bounded
            self.pools[bucket].push_back(buffer);
            self.stats.peak_pool_size = self.stats.peak_pool_size
                .max(self.pools.iter().map(|p| p.len()).sum());
        }
    }
    
    #[inline]
    fn bucket_for_capacity(capacity: usize) -> usize {
        match capacity {
            0..=8 => 0,
            9..=16 => 1,
            17..=32 => 2,
            33..=64 => 3,
            65..=128 => 4,
            129..=256 => 5,
            257..=512 => 6,
            _ => 7,
        }
    }
    
    pub fn stats(&self) -> &PoolStats {
        &self.stats
    }
}
```

### 2. RAII Buffer Guard

```rust
/// RAII wrapper - automatically returns buffer to pool on drop
pub struct BufferGuard {
    buffer: Option<Vec<Entity>>,
    pool: *mut QueryBufferPool,
    bucket: usize,
}

impl BufferGuard {
    fn new(buffer: Vec<Entity>, pool: &mut QueryBufferPool, bucket: usize) -> Self {
        Self {
            buffer: Some(buffer),
            pool: pool as *mut _,
            bucket,
        }
    }
    
    pub fn as_vec(&mut self) -> &mut Vec<Entity> {
        self.buffer.as_mut().unwrap()
    }
    
    pub fn as_slice(&self) -> &[Entity] {
        &self.buffer.as_ref().unwrap()
    }
}

impl Drop for BufferGuard {
    fn drop(&mut self) {
        if let Some(buffer) = self.buffer.take() {
            unsafe {
                (*self.pool).release(buffer, self.bucket);
            }
        }
    }
}
```

### 3. Integration with SpatialGrid

```rust
pub struct SpatialGrid {
    cells: HashMap<(i32, i32), Vec<Entity>>,
    entity_cells: HashMap<Entity, HashSet<(i32, i32)>>,
    buffer_pool: QueryBufferPool,  // NEW
}

impl SpatialGrid {
    pub fn query_circle(&mut self, pos: Vec2, radius: f32) -> BufferGuard {
        let cells = Self::cells_for_circle(pos, radius);
        
        // Estimate result size (typically 5-20 entities)
        let hint = cells.len().min(20);
        let mut guard = self.buffer_pool.acquire(hint);
        let result = guard.as_vec();
        
        let mut seen = HashSet::new();
        for cell in cells {
            if let Some(entities) = self.cells.get(&cell) {
                for &entity in entities {
                    if seen.insert(entity) {
                        result.push(entity);
                    }
                }
            }
        }
        
        guard  // Caller gets RAII guard, returns to pool when dropped
    }
    
    pub fn query_aabb(&mut self, min: Vec2, max: Vec2) -> BufferGuard {
        // Similar implementation
    }
}
```

---

## Acceptance Criteria

✅ `QueryBufferPool` with 8 capacity buckets  
✅ `BufferGuard` RAII wrapper (auto-return to pool)  
✅ `acquire()` reuses or allocates  
✅ `release()` returns to pool with bounds  
✅ Integrated into `SpatialGrid`  
✅ Zero allocations on hit (after warmup)  
✅ Benchmark: <1µs per query (vs 10µs allocating)  
✅ All 5 test cases pass  
✅ Compiles with no warnings  

---

## Tests

```rust
#[test]
fn test_buffer_pool_reuse() {
    let mut pool = QueryBufferPool::new();
    
    // Acquire two buffers
    let guard1 = pool.acquire(50);
    assert_eq!(pool.stats().allocations, 1);
    
    let guard2 = pool.acquire(50);
    assert_eq!(pool.stats().allocations, 2);
    
    // Drop them (return to pool)
    drop(guard1);
    drop(guard2);
    
    // Next acquire should reuse
    let guard3 = pool.acquire(50);
    assert_eq!(pool.stats().allocations, 2);  // No new allocation
    assert_eq!(pool.stats().reuses, 1);
}

#[test]
fn test_buffer_capacity_buckets() {
    let mut pool = QueryBufferPool::new();
    
    let g1 = pool.acquire(5);   // Bucket 0
    let g2 = pool.acquire(20);  // Bucket 2
    let g3 = pool.acquire(100); // Bucket 4
    
    assert_eq!(pool.stats().allocations, 3);
    drop(g1);
    drop(g2);
    drop(g3);
    
    // Reacquire same buckets
    pool.acquire(8);   // Should reuse bucket 0
    pool.acquire(25);  // Should reuse bucket 2
    pool.acquire(120); // Should reuse bucket 4
    
    assert_eq!(pool.stats().reuses, 3);
}

#[test]
fn test_zero_alloc_hot_path() {
    let mut pool = QueryBufferPool::new();
    
    // Warmup
    for _ in 0..10 {
        let g = pool.acquire(32);
        drop(g);
    }
    
    let before_allocs = pool.stats().allocations;
    
    // Hot path (100 queries)
    for _ in 0..100 {
        let g = pool.acquire(32);
        drop(g);
    }
    
    // Should be zero new allocations
    assert_eq!(pool.stats().allocations, before_allocs);
}

#[test]
fn test_pool_bounds() {
    let mut pool = QueryBufferPool::new();
    
    // Try to add 100 buffers to one bucket
    let mut guards = Vec::new();
    for _ in 0..100 {
        guards.push(pool.acquire(64));
    }
    
    // All should get individual allocations (pool bounded at 8)
    drop(guards);
    
    // Pool should only keep 8
    assert!(pool.pools[3].len() <= 8);
}

#[test]
fn test_benchmark_query_with_pool() {
    use std::time::Instant;
    
    let mut pool = QueryBufferPool::new();
    
    // Warmup
    for _ in 0..100 {
        let g = pool.acquire(50);
        drop(g);
    }
    
    // Benchmark 1000 acquisitions
    let start = Instant::now();
    for _ in 0..1000 {
        let g = pool.acquire(50);
        drop(g);
    }
    let elapsed = start.elapsed();
    
    let per_query = elapsed.as_micros() as f32 / 1000.0;
    println!("Per-query time: {:.2}µs", per_query);
    
    // Should be <1µs (VERY fast after pooling)
    assert!(per_query < 1.0, "Per-query time: {:.2}µs", per_query);
}
```

---

## Verification Command

```bash
cargo test test_buffer_pool_reuse
cargo test test_zero_alloc_hot_path
cargo test test_benchmark_query_with_pool
cargo run --release --example integrated_demo -- --stress-test
# Should show <1µs per query in profiler
```

---

## Judge Check

```bash
./verify_solive_delivery.ps1 JOB_QUERY_POOL
```

Expected: ✅ Zero allocation hot path, <1µs per query, pool reuse working


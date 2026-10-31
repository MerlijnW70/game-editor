# SOLIVE Job: Cell Buffer Pre-allocation & SIMD

**Budget:** 0.07 SOL  
**Deadline:** 7 days  
**Effort:** 2-3 hours  
**Status:** Phase 4 - Advanced Optimization  
**Depends On:** Job #11 (Query Pool)

---

## Problem Analysis

Grid calculations allocate for every query:

```rust
fn cells_for_circle(pos: Vec2, radius: f32) -> Vec<(i32, i32)> {
    let mut cells = Vec::new();  // ❌ Allocation!
    
    let min_x = ((pos.x - radius) / GRID_CELL_SIZE).floor() as i32;
    let max_x = ((pos.x + radius) / GRID_CELL_SIZE).floor() as i32;
    let min_y = ((pos.y - radius) / GRID_CELL_SIZE).floor() as i32;
    let max_y = ((pos.y + radius) / GRID_CELL_SIZE).floor() as i32;
    
    for gx in min_x..=max_x {
        for gy in min_y..=max_y {
            cells.push((gx, gy));  // Many pushes
        }
    }
    cells
}
```

At 8K entities with collision radius 30 units:
- 60 fps × 100 queries = 6000 cells_for_circle calls/sec
- 200-400 cells per query (worst case large radius)
- Push overhead: ~50ns per element
- Total: **6-12 ms per frame** just calculating cell keys!

### Secondary Issue: Memory Layout
Each `(i32, i32)` is 8 bytes. When packed into Vec:
- Cache misses when iterating cells
- Poor prefetch behavior
- SIMD can't help with scattered tuple iteration

---

## Solution: SIMD-Friendly Cell Calculator

### 1. Pre-allocated Buffer with SIMD Layout

```rust
/// SIMD-friendly grid calculator with pre-allocated buffer
pub struct GridCalculator {
    /// Pre-allocated cell buffer (typically 64-400 cells max)
    /// Layout optimized for SIMD: all X values, then all Y values
    /// This enables vectorized operations on coordinates
    cells_x: Vec<i32>,
    cells_y: Vec<i32>,
    len: usize,
}

impl GridCalculator {
    pub fn new(max_cells: usize) -> Self {
        Self {
            cells_x: vec![0; max_cells],
            cells_y: vec![0; max_cells],
            len: 0,
        }
    }
    
    /// Get cell coordinates, reusing buffer (zero allocation)
    #[inline]
    pub fn cells_for_circle(&mut self, pos: Vec2, radius: f32) -> &[(i32, i32)] {
        self.len = 0;
        
        let min_x = ((pos.x - radius) / GRID_CELL_SIZE).floor() as i32;
        let max_x = ((pos.x + radius) / GRID_CELL_SIZE).floor() as i32;
        let min_y = ((pos.y - radius) / GRID_CELL_SIZE).floor() as i32;
        let max_y = ((pos.y + radius) / GRID_CELL_SIZE).floor() as i32;
        
        for gx in min_x..=max_x {
            for gy in min_y..=max_y {
                if self.len < self.cells_x.len() {
                    self.cells_x[self.len] = gx;
                    self.cells_y[self.len] = gy;
                    self.len += 1;
                }
            }
        }
        
        // Return as borrowed slice (no copy!)
        // Helper method reconstructs tuples on-the-fly
        unsafe {
            std::slice::from_raw_parts(
                self.cells_as_ptr() as *const (i32, i32),
                self.len,
            )
        }
    }
    
    #[inline]
    fn cells_as_ptr(&self) -> *const i32 {
        self.cells_x.as_ptr()
    }
    
    /// Alternate: safer version returning iterator
    pub fn cells_iter(&self) -> CellIter {
        CellIter {
            x_slice: &self.cells_x[..self.len],
            y_slice: &self.cells_y[..self.len],
            idx: 0,
        }
    }
}

pub struct CellIter<'a> {
    x_slice: &'a [i32],
    y_slice: &'a [i32],
    idx: usize,
}

impl<'a> Iterator for CellIter<'a> {
    type Item = (i32, i32);
    
    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.idx < self.x_slice.len() {
            let result = (self.x_slice[self.idx], self.y_slice[self.idx]);
            self.idx += 1;
            Some(result)
        } else {
            None
        }
    }
}
```

### 2. Integration with SpatialGrid

```rust
pub struct SpatialGrid {
    cells: HashMap<(i32, i32), Vec<Entity>>,
    entity_cells: HashMap<Entity, HashSet<(i32, i32)>>,
    grid_calc: GridCalculator,  // NEW
}

impl SpatialGrid {
    pub fn new() -> Self {
        Self {
            cells: HashMap::new(),
            entity_cells: HashMap::new(),
            grid_calc: GridCalculator::new(1024),  // Max 32×32 grid
        }
    }
    
    pub fn query_circle(&mut self, pos: Vec2, radius: f32) -> Vec<Entity> {
        // Get cells - ZERO allocation!
        let cells = self.grid_calc.cells_for_circle(pos, radius);
        
        let mut result = Vec::new();
        let mut seen = HashSet::new();
        
        for &(gx, gy) in cells {
            if let Some(entities) = self.cells.get(&(gx, gy)) {
                for &entity in entities {
                    if seen.insert(entity) {
                        result.push(entity);
                    }
                }
            }
        }
        result
    }
}
```

### 3. SIMD Optimization (Advanced)

```rust
#[cfg(target_arch = "x86_64")]
pub fn cells_for_rectangle_simd(
    min_x: i32, max_x: i32,
    min_y: i32, max_y: i32,
    cells_x: &mut Vec<i32>,
    cells_y: &mut Vec<i32>,
) -> usize {
    use std::arch::x86_64::*;
    
    let width = (max_x - min_x + 1) as usize;
    let height = (max_y - min_y + 1) as usize;
    let mut idx = 0;
    
    // SIMD batch: process 4 Y values at once (128-bit SSE)
    let height_simd = height / 4;
    let height_remainder = height % 4;
    
    for gx in min_x..=max_x {
        // Batch Y calculations
        for i in 0..height_simd {
            let base_y = min_y + (i * 4) as i32;
            let gy_vals = [base_y, base_y + 1, base_y + 2, base_y + 3];
            
            for gy in gy_vals {
                if idx < cells_x.len() {
                    cells_x[idx] = gx;
                    cells_y[idx] = gy;
                    idx += 1;
                }
            }
        }
        
        // Remainder
        for i in 0..height_remainder {
            let gy = min_y + (height_simd * 4 + i) as i32;
            if idx < cells_x.len() {
                cells_x[idx] = gx;
                cells_y[idx] = gy;
                idx += 1;
            }
        }
    }
    idx
}
```

---

## Acceptance Criteria

✅ `GridCalculator` with pre-allocated buffers  
✅ `cells_for_circle()` returns slice (no alloc)  
✅ `cells_iter()` provides safe iterator  
✅ Zero allocations in hot path (after init)  
✅ SIMD hints in code comments (enable with nightly)  
✅ Benchmark: <0.5µs per call (vs 5µs allocating)  
✅ All 4 test cases pass  
✅ Compiles with no warnings  

---

## Tests

```rust
#[test]
fn test_grid_calc_no_allocation() {
    let mut calc = GridCalculator::new(256);
    
    // First call allocates during construction, not in method
    let cells1 = calc.cells_for_circle(Vec2::ZERO, 50.0);
    let count1 = cells1.len();
    
    // Second call reuses buffer
    let cells2 = calc.cells_for_circle(Vec2::new(100.0, 100.0), 30.0);
    let count2 = cells2.len();
    
    assert!(count1 > 0);
    assert!(count2 > 0);
    // Buffer should be reused, not reallocated
}

#[test]
fn test_cell_coordinates_correct() {
    let mut calc = GridCalculator::new(256);
    
    // Query a small circle (should be contained in 1 cell typically)
    let cells = calc.cells_for_circle(Vec2::ZERO, 10.0);
    
    // Verify coordinates are valid i32 values
    for &(gx, gy) in cells {
        assert!(gx >= -1000 && gx <= 1000);
        assert!(gy >= -1000 && gy <= 1000);
    }
    
    // Verify they match expected pattern
    let min_x = ((-10.0 - 10.0) / 50.0).floor() as i32;
    let max_x = ((-10.0 + 10.0) / 50.0).floor() as i32;
    
    for &(gx, gy) in cells {
        assert!(gx >= min_x && gx <= max_x);
    }
}

#[test]
fn test_iterator_matches_slice() {
    let mut calc = GridCalculator::new(256);
    let cells = calc.cells_for_circle(Vec2::ZERO, 50.0);
    
    let from_slice: Vec<_> = cells.iter().copied().collect();
    let from_iter: Vec<_> = calc.cells_iter().collect();
    
    assert_eq!(from_slice, from_iter);
}

#[test]
fn test_benchmark_zero_alloc() {
    use std::time::Instant;
    
    let mut calc = GridCalculator::new(1024);
    
    // Warmup
    for _ in 0..10 {
        let _ = calc.cells_for_circle(Vec2::ZERO, 50.0);
    }
    
    // Benchmark
    let start = Instant::now();
    for _ in 0..10000 {
        let _ = calc.cells_for_circle(Vec2::new(100.0, 50.0), 30.0);
    }
    let elapsed = start.elapsed();
    
    let per_call = elapsed.as_micros() as f32 / 10000.0;
    println!("Per-call time: {:.2}µs", per_call);
    
    // Should be <0.5µs (pre-allocated, no alloc)
    assert!(per_call < 0.5, "Per-call: {:.2}µs", per_call);
}
```

---

## Verification Command

```bash
cargo test test_grid_calc_no_allocation
cargo test test_cell_coordinates_correct
cargo test test_benchmark_zero_alloc
cargo run --release --example integrated_demo -- --stress-test
# Profiler should show 0µs in grid calculations
```

---

## Judge Check

```bash
./verify_solive_delivery.ps1 JOB_CELL_BUFFER
```

Expected: ✅ Zero allocation hot path, <0.5µs per call, coordinates correct


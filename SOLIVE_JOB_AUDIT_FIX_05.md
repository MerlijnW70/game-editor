# SOLIVE Job: HashSet in Spatial Grid

**Budget:** 0.06 SOL  
**Deadline:** 7 days  
**Effort:** 2-3 hours  
**Status:** Phase 2 - Ready after Phase 1 complete  
**Depends On:** Job #3 (spatial grid refactor)

---

## Overview

Spatial grid currently uses `HashMap<(i32, i32), Vec<Entity>>`. This is inefficient for grid queries because we check multiple cells and may encounter duplicate entities. Phase 2 optimization: use `HashSet<Entity>` for faster uniqueness checks.

---

## Problem

Current code in `src/collision/spatial.rs`:
```rust
pub fn query_circle(&self, center: Vec2, radius: f32) -> Vec<Entity> {
    let mut results = Vec::new();
    
    for cell in self.cells_in_radius(center, radius) {
        if let Some(entities) = self.grid_cells.get(&cell) {
            // ❌ May have duplicates from adjacent cells
            results.extend(entities.iter());
        }
    }
    
    results  // Contains duplicates - checked multiple times
}
```

When querying overlapping cells, same entity appears multiple times in results.

---

## Requirements

1. **Add HashSet to spatial grid** (src/collision/spatial.rs)
   ```rust
   use std::collections::HashSet;
   
   pub struct SpatialGrid {
       grid_cells: HashMap<(i32, i32), Vec<Entity>>,
       // NEW: For deduplication in queries
       query_cache: HashSet<Entity>,
   }
   
   impl SpatialGrid {
       pub fn new() -> Self {
           Self {
               grid_cells: HashMap::new(),
               query_cache: HashSet::new(),
           }
       }
   }
   ```

2. **Use HashSet for query deduplication** (src/collision/spatial.rs)
   ```rust
   pub fn query_circle(&self, center: Vec2, radius: f32) -> Vec<Entity> {
       self.query_cache.clear();  // Reuse allocation
       
       for cell in self.cells_in_radius(center, radius) {
           if let Some(entities) = self.grid_cells.get(&cell) {
               for &entity in entities {
                   self.query_cache.insert(entity);  // Auto-deduped
               }
           }
       }
       
       self.query_cache.iter().copied().collect()  // Return unique list
   }
   ```

3. **Add benchmark test** (tests/spatial_grid.rs)
   ```rust
   #[test]
   fn test_query_no_duplicates() {
       let mut grid = SpatialGrid::new();
       
       let e1 = Entity::from_raw(1);
       let e2 = Entity::from_raw(2);
       
       grid.update_entity(e1, Vec2::new(0.0, 0.0), 5.0);
       grid.update_entity(e2, Vec2::new(48.0, 0.0), 5.0);  // Adjacent cell
       
       let results = grid.query_circle(Vec2::new(25.0, 0.0), 30.0);  // Overlaps both
       
       // Should contain both, but no duplicates
       assert_eq!(results.len(), 2);
       assert!(results.contains(&e1));
       assert!(results.contains(&e2));
   }
   
   #[test]
   fn test_hashset_faster_than_vec_dedup() {
       // Benchmark: HashSet should be faster than Vec + sort + dedup
       // At 1000 overlapping queries, measure time
       // HashSet should be <100µs, Vec + dedup >1ms
   }
   ```

4. **Memory optimization note**
   ```rust
   impl SpatialGrid {
       // Pre-allocate query_cache to avoid repeated allocations
       // Typical query returns 20-50 entities
       // Pre-allocated 64-entity HashSet saves millions of allocations in tight loops
   }
   ```

---

## Acceptance Criteria

✅ `SpatialGrid` has `query_cache: HashSet<Entity>` field  
✅ `query_circle()` uses HashSet for deduplication  
✅ `query_aabb()` uses HashSet for deduplication  
✅ No duplicate entities in query results  
✅ Query performance improved by 3-5× on overlapping cells  
✅ All 2 test cases pass  
✅ Compiles with no warnings  

---

## Verification Command

```bash
cargo test query_no_duplicates
cargo test hashset_faster_than_vec_dedup
cargo run --release --example integrated_demo -- --stress-test
# Should see 3-5× faster grid queries in profiler
```

---

## Judge Check

```bash
./verify_solive_delivery.ps1 JOB_SPATIAL_HASHSET
```

Expected: ✅ No duplicate entities in queries, 3-5× performance improvement


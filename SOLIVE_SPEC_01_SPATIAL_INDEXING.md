# Task: Upgrade Spatial Indexing to Quadtree

## Objective
Replace the current grid-based collision spatial partitioning with a **Quadtree** implementation for O(log n) neighbor queries, enabling scalable collision detection at 300+ entities.

## Why This Matters
- Current: Linear grid queries, fine for 100 entities
- Target: Quadtree enables 300+ entities without performance cliff
- This is an **architectural decision** — adding later requires major refactoring

## Technical Requirements

### 1. Quadtree Implementation (`src/collision/spatial.rs` refactor)
- **Node structure**: Internal nodes (4 children) + leaf nodes (entity list)
- **Bounds**: Dynamic AABB bounding each quadrant
- **Capacity**: Max 4 entities per leaf before split
- **Operations**: 
  - `insert(entity, bounds)` → O(log n)
  - `query(region)` → returns entities in AABB, O(k) where k = results
  - `update(entity, old_bounds, new_bounds)` → remove + re-insert
  - `remove(entity)` → O(log n)

### 2. Integration with ECS
- Rebuild quadtree each frame (simple; optimization for later)
- Query method: `fn query_region(&self, bounds: &AABB) -> Vec<Entity>`
- Use in collision broad-phase: only test entities in overlapping nodes

### 3. Performance Targets
- **Tree update**: < 0.3ms per frame (100 entities)
- **Query performance**: < 0.2ms per frame (50 collision checks vs 10,000 all-pairs)
- **Memory**: < 1MB for 300 entities + tree nodes
- **Proof**: Benchmark with 300 entities, measure frame time breakdown

### 4. Example & Tests
- Create `examples/quadtree_demo.rs` spawning 300 entities, measuring query performance
- Unit tests: insertion, removal, query correctness, tree balance
- Comparison benchmark: grid vs quadtree at 100/200/300 entities

## Deliverable Format
- Refactored `src/collision/spatial.rs` with Quadtree impl
- Updated `src/systems/collision.rs` to use quadtree queries
- Example: `examples/quadtree_demo.rs` showing 300 entities at 60+ FPS
- Tests: `tests/quadtree.rs` covering insertion, query, removal
- Documentation: `QUADTREE_IMPLEMENTATION.md` explaining design choices

## Acceptance Criteria
1. Quadtree compiles, no warnings
2. All tests pass (insertion, query, removal)
3. 300-entity demo runs at 60+ FPS
4. Query time measured as < 0.2ms per frame
5. Tree memory footprint < 1MB
6. Code integrates cleanly with existing collision system
7. Git commit with clear message

## Payment
**0.08 SOL**

## Timeline
- **Estimated**: 6-8 hours
- **Deadline**: 7 days from job posting

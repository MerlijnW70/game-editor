# Task 3: Collision Detection & Physics System

## Objective
Implement efficient spatial partitioning and collision detection for 100+ simultaneous entities.

## Technical Requirements

### 1. Spatial Partitioning
Implement ONE of:
- **Quadtree** (recommended for dynamic entities)
- **Grid-based spatial hash** (simpler, good for uniform density)
- **BVH (Bounding Volume Hierarchy)** (advanced, best performance)

Requirements:
- Dynamic updates as entities move
- Query capabilities (point query, radius query, AABB query)
- Rebuild/rebalance strategy for moving entities
- Performance: < 0.5ms per frame for spatial structure updates with 100 entities

### 2. Collision Detection

#### Broad Phase
- Use spatial partitioning to identify potential collisions
- Only test pairs that are spatially close
- Achieve < 50 collision checks per frame for 100 entities (not all-pairs O(n²))

#### Narrow Phase
- Circle/Circle collision (most entities use collision radius)
- Support for rectangle collision (if needed for obstacles)
- Return collision info: contact point, normal, penetration depth
- Handle overlap resolution (entities don't pass through each other)

### 3. Collision Response System
- **Knockback**: Apply velocity separation when entities collide
- **Damage on Contact**: Projectile/enemy hits trigger damage events
- **Movement Blocking**: Entities don't phase through obstacles/walls
- **Team Filtering**: Player doesn't collide with allies, enemies don't damage each other (configurable)

### 4. Performance Targets
- Broad phase: < 0.3ms per frame (100 entities)
- Narrow phase: < 0.2ms per frame
- Total collision system: < 1ms per frame
- 60+ FPS maintained with 100+ entities + physics

### 5. Query Interfaces
Expose collision queries:
- `raycast(origin, direction) -> Option<(entity, hit_point)>`
- `circle_overlap(center, radius) -> Vec<Entity>`
- `get_collisions_for_entity(entity) -> Vec<Collision>`

## Deliverable Format
- Integrate into Bevy ECS project (Task 1 or standalone)
- Spatial partitioning in `src/collision/spatial.rs`
- Collision detection in `src/collision/detection.rs`
- Collision response in `src/systems/collision.rs`
- Example scene in `examples/collision_demo.rs` showing:
  - 100 entities moving around
  - Collisions being detected and responded to
  - No clipping/phasing through objects
  - Performance metrics (collision time per frame)

## Acceptance Criteria
1. Spatial structure updates < 0.5ms per frame at 100 entities
2. Collision detection finds all actual collisions (no false negatives)
3. Collision response prevents entity overlap
4. Knockback works (entities push away from collisions)
5. Team filtering works (player doesn't collide with allies)
6. Performance: 60+ FPS at 100 entities with collision system active
7. Code is well-documented (collision complex, needs clear explanation)
8. No memory leaks or excessive allocations per frame

## Debug Features
- Render collision circles/boxes for debugging
- Log collision events with entity IDs
- Show broad phase results (spatial partitions being tested)
- Frame time breakdown showing collision system % of frame

## Payment & Timeline
- Payment: 0.05 SOL
- Estimated time: 8-12 hours (most complex system)
- Deliverable: Working collision system integrated with ECS

## Judge Verification
1. Run `examples/collision_demo.rs`
2. Spawn 100 entities and verify no clipping/phasing
3. Observe knockback effect (entities push apart on collision)
4. Check collision event logs
5. Measure frame time (collision portion should be < 1ms)
6. Examine spatial structure implementation for efficiency
7. Verify team collision filtering works correctly
8. Test edge cases: corner collisions, stacked entities, fast-moving entities

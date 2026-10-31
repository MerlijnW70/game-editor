# SOLIVE Job: Spatial Grid & Bounds Checking

**Budget:** 0.14 SOL  
**Deadline:** 7 days  
**Effort:** 3-5 hours  
**Status:** Open for claims

---

## Spec

Fix critical safety issues in spatial grid collision detection:
1. Negative coordinates cause integer overflow
2. Collision radius not validated against grid cell size
3. Missing bounds checking on critical values

### Problem 1: Negative Coordinate Wrapping

Current code in `src/collision/spatial.rs`:
```rust
fn world_to_grid(&self, pos: Vec2) -> (usize, usize) {
    let gx = (pos.x / CELL_SIZE) as usize;  // ❌ Negative becomes huge!
    let gy = (pos.y / CELL_SIZE) as usize;
    (gx, gy)
}
```

When `pos.x = -10.0`: `(-10 as usize)` wraps to ~4 billion, causing wrong cell lookup.

### Problem 2: Radius Validation

No check that `collision_radius < CELL_SIZE`. Entities larger than cells are missed.

### Requirements

1. **Change grid indexing to signed coordinates** (src/collision/spatial.rs)
   ```rust
   // From: HashMap<(usize, usize), Vec<Entity>>
   // To:
   grid_cells: HashMap<(i32, i32), Vec<Entity>>,
   
   fn world_to_grid(&self, pos: Vec2) -> (i32, i32) {
       let gx = (pos.x / CELL_SIZE).floor() as i32;
       let gy = (pos.y / CELL_SIZE).floor() as i32;
       (gx, gy)
   }
   ```

2. **Add radius validation** (src/collision/spatial.rs)
   ```rust
   pub fn update_entity(&mut self, entity: Entity, pos: Vec2, radius: f32) {
       assert!(radius > 0.0, "Collision radius must be positive");
       assert!(radius <= CELL_SIZE * 2.0, 
           "Collision radius {} exceeds safe grid cell size {}", 
           radius, CELL_SIZE * 2.0);
       
       // ... existing logic
   }
   ```

3. **Add bounds checking in spawning** (src/systems/spawning.rs)
   ```rust
   pub fn spawn_enemies(
       // ... existing params
   ) {
       const MAX_ENTITIES: usize = 8192;
       if all_entities.iter().count() >= MAX_ENTITIES {
           warn!("Entity cap {} reached, skipping spawns", MAX_ENTITIES);
           return;
       }
   }
   ```

4. **Add bounds checking for velocity** (src/systems/movement.rs)
   ```rust
   pub fn movement(
       mut query: Query<(&mut Transform2D, &Velocity)>,
   ) {
       const MAX_VELOCITY: f32 = 500.0;  // Units per second
       
       for (mut transform, velocity) in query.iter_mut() {
           let speed = velocity.0.length();
           if speed > MAX_VELOCITY {
               warn!("Velocity {} exceeds max {}", speed, MAX_VELOCITY);
           }
           transform.position += velocity.0 * time.delta_seconds();
       }
   }
   ```

5. **Add bounds checking for damage** (src/systems/health.rs)
   ```rust
   pub fn apply_damage(
       mut damage_events: MessageReader<DamageEvent>,
       mut death_events: MessageWriter<DeathEvent>,
       mut query: Query<&mut Health>,
   ) {
       for event in damage_events.read() {
           let Ok(mut health) = query.get_mut(event.target) else { continue; };
           
           // Validate damage amount
           if event.amount <= 0.0 {
               warn!("Ignoring invalid damage amount: {}", event.amount);
               continue;
           }
           
           // Apply damage
           let was_alive = !health.is_dead();
           health.apply_damage(event.amount);
           
           if was_alive && health.is_dead() {
               death_events.write(DeathEvent { entity: event.target });
           }
       }
   }
   ```

6. **Add tests** (tests/spatial_grid.rs)
   ```rust
   #[test]
   fn test_negative_coordinates() {
       let mut grid = SpatialGrid::new();
       let e1 = Entity::from_raw(1);
       
       // Should not panic or wrap incorrectly
       grid.update_entity(e1, Vec2::new(-100.0, -50.0), 5.0);
       
       let near = grid.query_circle(Vec2::new(-100.0, -50.0), 10.0);
       assert!(near.contains(&e1));
   }
   
   #[test]
   fn test_radius_validation() {
       let mut grid = SpatialGrid::new();
       let e1 = Entity::from_raw(1);
       
       // Should panic or error on radius too large
       // Radius > 100 (2 × cell_size) should fail
       let result = std::panic::catch_unwind(|| {
           grid.update_entity(e1, Vec2::ZERO, 200.0);
       });
       assert!(result.is_err());
   }
   
   #[test]
   fn test_velocity_clamp() {
       let velocity = Velocity(Vec2::new(1000.0, 1000.0));
       assert!(velocity.0.length() > 500.0);  // Exceeds max
   }
   
   #[test]
   fn test_damage_validation() {
       // Negative damage should be ignored
       // Test by checking no DeathEvent emitted on negative damage
   }
   ```

---

## Acceptance Criteria

✅ Grid uses `HashMap<(i32, i32), Vec<Entity>>` (signed coordinates)  
✅ `world_to_grid()` uses `.floor() as i32` (handles negatives correctly)  
✅ `update_entity()` validates `radius > 0` and `radius <= 2 * CELL_SIZE`  
✅ Movement system clamps velocity to MAX_VELOCITY  
✅ Health system ignores damage <= 0  
✅ Spawning respects entity cap and logs warning  
✅ All 4 test cases pass  
✅ Negative coordinates no longer wrap incorrectly  
✅ Compiles with no warnings  

---

## Verification Command

```bash
cargo test negative_coordinates
cargo test radius_validation
cargo test velocity_clamp
cargo test damage_validation
cargo run --example integrated_demo -- --stress-test  # Spawn at negative coords
```

---

## Judge Check

```bash
./verify_solive_delivery.ps1 JOB_BOUNDS_FIX
```

Expected: ✅ Negative coordinates handled correctly, no overflow, bounds enforced

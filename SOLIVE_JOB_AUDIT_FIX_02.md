# SOLIVE Job: Entity ID & Collision Attribution Fix

**Budget:** 0.13 SOL  
**Deadline:** 7 days  
**Effort:** 3-4 hours  
**Status:** Open for claims

---

## Spec

Fix critical bug where collisions are attributed to entity ID 0 instead of actual colliding entities. This breaks damage attribution in the collision → damage pipeline.

### Problem

In `src/collision/detection.rs:68-69`, `ContactInfo` is created with placeholder entity IDs:

```rust
Some(ContactInfo {
    entity_a: Entity::from_bits(0),  // ❌ WRONG: Always 0
    entity_b: Entity::from_bits(0),  // ❌ WRONG: Always 0
    contact_point,
    normal,
    penetration,
})
```

When collision response applies damage, it damages entity 0 instead of actual entities.

### Requirements

1. **Update `test_circle_collision()` signature** (src/collision/detection.rs)
   ```rust
   pub fn test_circle_collision(
       entity_a: Entity,              // NEW: Actual entity IDs
       pos_a: Vec2,
       radius_a: f32,
       entity_b: Entity,              // NEW: Actual entity IDs
       pos_b: Vec2,
       radius_b: f32,
   ) -> Option<ContactInfo>
   ```

2. **Populate ContactInfo with real entity IDs** (src/collision/detection.rs)
   ```rust
   Some(ContactInfo {
       entity_a,  // ✅ Use parameter
       entity_b,  // ✅ Use parameter
       contact_point,
       normal,
       penetration,
   })
   ```

3. **Update all callers** in collision detection systems (src/systems/collision.rs)
   - Find all calls to `test_circle_collision()`
   - Pass actual entity IDs from the query loop
   
   **Example:**
   ```rust
   for (entity_a, pos_a, coll_a) in query.iter() {
       for (entity_b, pos_b, coll_b) in query.iter() {
           if let Some(contact) = test_circle_collision(
               entity_a, pos_a, coll_a.radius,   // NEW: entity IDs
               entity_b, pos_b, coll_b.radius,
           ) {
               // Process contact with real entity IDs
           }
       }
   }
   ```

4. **Update `test_circle_aabb_collision()` similarly** (src/collision/detection.rs)
   ```rust
   pub fn test_circle_aabb_collision(
       entity: Entity,  // NEW
       circle_pos: Vec2,
       circle_radius: f32,
       box_min: Vec2,
       box_max: Vec2,
   ) -> Option<ContactInfo>
   ```

5. **Add tests** (tests/collision_system.rs)
   ```rust
   #[test]
   fn test_collision_entity_ids() {
       let e1 = Entity::from_raw(100);
       let e2 = Entity::from_raw(200);
       
       let contact = test_circle_collision(
           e1, Vec2::ZERO, 10.0,
           e2, Vec2::new(15.0, 0.0), 10.0,
       ).unwrap();
       
       assert_eq!(contact.entity_a, e1);  // ✅ Correct ID
       assert_eq!(contact.entity_b, e2);  // ✅ Correct ID
   }
   
   #[test]
   fn test_collision_no_longer_entity_zero() {
       let result = test_circle_collision(
           Entity::from_raw(1), Vec2::ZERO, 10.0,
           Entity::from_raw(2), Vec2::new(5.0, 0.0), 10.0,
       );
       
       assert!(result.is_some());
       let contact = result.unwrap();
       assert_ne!(contact.entity_a, Entity::from_bits(0));
       assert_ne!(contact.entity_b, Entity::from_bits(0));
   }
   ```

---

## Acceptance Criteria

✅ `test_circle_collision()` accepts entity IDs as parameters  
✅ `test_circle_aabb_collision()` accepts entity ID as parameter  
✅ ContactInfo populated with actual entity IDs (not zeros)  
✅ All callers in collision systems updated  
✅ All 2 test cases pass  
✅ Damage is attributed to correct entities (verify in integrated_demo)  
✅ Compiles with no warnings  

---

## Verification Command

```bash
cargo test collision_entity_ids
cargo run --example integrated_demo  # Play, verify damage attribution
```

---

## Judge Check

```bash
./verify_solive_delivery.ps1 JOB_COLLISION_FIX
```

Expected: ✅ Entity IDs correctly assigned, no entity 0 damage attribution
